use super::{store::TokenStore, TokenProvider};
use crate::error::LinearMcpError;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, instrument, warn};

const ACCESS_TOKEN_KEY: &str = "linear_oauth_access_token";
const REFRESH_TOKEN_KEY: &str = "linear_oauth_refresh_token";
/// Refresh the access token this many seconds before it actually expires
/// to avoid races at the boundary.
const REFRESH_BUFFER_SECS: i64 = 60;

/// An OAuth 2.1 token pair with expiry metadata.
#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub access_token: Secret<String>,
    pub refresh_token: Option<Secret<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl OAuthToken {
    pub fn needs_refresh(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() + Duration::seconds(REFRESH_BUFFER_SECS) >= exp,
            None => false,
        }
    }
}

/// OAuth token provider that accepts externally-obtained tokens and
/// handles automatic refresh. Browser login / PKCE is NOT owned here —
/// the caller constructs this with an already-exchanged token pair.
pub struct OAuthTokenProvider<S: TokenStore> {
    token: Mutex<OAuthToken>,
    store: Arc<S>,
    token_endpoint: url::Url,
    client_id: String,
    http: reqwest::Client,
}

impl<S: TokenStore> OAuthTokenProvider<S> {
    pub fn new(
        initial_token: OAuthToken,
        store: Arc<S>,
        token_endpoint: url::Url,
        client_id: impl Into<String>,
        http: reqwest::Client,
    ) -> Self {
        Self {
            token: Mutex::new(initial_token),
            store,
            token_endpoint,
            client_id: client_id.into(),
            http,
        }
    }

    #[instrument(skip(self), name = "oauth.refresh")]
    async fn refresh(&self, refresh_token: &Secret<String>) -> Result<OAuthToken, LinearMcpError> {
        debug!("Refreshing OAuth access token");

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        let resp = self
            .http
            .post(self.token_endpoint.clone())
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.expose_secret().as_str()),
                ("client_id", self.client_id.as_str()),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(LinearMcpError::TokenRefresh {
                source: format!("token endpoint returned {status}: {body}").into(),
            });
        }

        let raw: TokenResponse = resp.json().await?;
        let expires_at = raw
            .expires_in
            .map(|secs| Utc::now() + Duration::seconds(secs as i64));

        let new_token = OAuthToken {
            access_token: Secret::new(raw.access_token),
            refresh_token: raw.refresh_token.map(Secret::new).or_else(|| {
                // Keep the old refresh token if the server doesn't rotate it.
                Some(refresh_token.clone())
            }),
            expires_at,
        };

        // Persist asynchronously — log but don't fail the request if storage errors.
        if let Err(e) = self
            .store
            .save(ACCESS_TOKEN_KEY, new_token.access_token.clone())
            .await
        {
            warn!(error = %e, "Failed to persist refreshed access token");
        }

        Ok(new_token)
    }
}

impl<S: TokenStore> std::fmt::Debug for OAuthTokenProvider<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthTokenProvider")
            .field("client_id", &self.client_id)
            .field("token_endpoint", &self.token_endpoint)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl<S: TokenStore> TokenProvider for OAuthTokenProvider<S> {
    #[instrument(skip(self), name = "oauth.token")]
    async fn token(&self) -> Result<Secret<String>, LinearMcpError> {
        let mut guard = self.token.lock().await;

        if guard.needs_refresh() {
            match &guard.refresh_token.clone() {
                Some(rt) => {
                    let refreshed = self.refresh(rt).await?;
                    *guard = refreshed;
                }
                None => {
                    return Err(LinearMcpError::Auth {
                        reason: "Token expired and no refresh token available".into(),
                    });
                }
            }
        }

        Ok(guard.access_token.clone())
    }
}
