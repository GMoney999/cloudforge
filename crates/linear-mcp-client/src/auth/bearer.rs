use super::TokenProvider;
use crate::error::LinearMcpError;
use async_trait::async_trait;
use secrecy::Secret;

/// Static bearer token provider. Wraps a `Secret<String>` so the raw
/// token value is never accidentally `Debug`-printed or logged.
#[derive(Clone)]
pub struct BearerTokenProvider {
    token: Secret<String>,
}

impl BearerTokenProvider {
    /// Construct from any `Into<String>`. The value is immediately
    /// wrapped in `Secret` to prevent leakage.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Secret::new(token.into()),
        }
    }

    /// Convenience: read token from an environment variable.
    pub fn from_env(var: &str) -> Result<Self, LinearMcpError> {
        let raw = std::env::var(var).map_err(|_| LinearMcpError::Auth {
            reason: format!("environment variable `{var}` not set"),
        })?;
        Ok(Self::new(raw))
    }
}

impl std::fmt::Debug for BearerTokenProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerTokenProvider")
            .field("token", &"[redacted]")
            .finish()
    }
}

#[async_trait]
impl TokenProvider for BearerTokenProvider {
    async fn token(&self) -> Result<Secret<String>, LinearMcpError> {
        Ok(self.token.clone())
    }
}
