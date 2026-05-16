// src/transport/streamable_http.rs

use super::{McpRequest, McpResponse, McpTransport};
use crate::{auth::TokenProvider, error::LinearMcpError};
use async_trait::async_trait;
use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tracing::{debug, instrument, warn};

/// Configuration for the Streamable HTTP transport.
#[derive(Debug, Clone)]
pub struct StreamableHttpConfig {
    /// Linear's MCP endpoint, e.g. `https://mcp.linear.app/mcp`
    pub endpoint: Url,
    /// Per-request HTTP timeout (independent of session timeout).
    pub request_timeout: Duration,
    /// Optional maximum lifetime of a long-lived session.
    pub session_timeout: Option<Duration>,
}

impl Default for StreamableHttpConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://mcp.linear.app/mcp"
                .parse()
                .expect("hard-coded URL is valid"),
            request_timeout: Duration::from_secs(30),
            session_timeout: Some(Duration::from_secs(3600)),
        }
    }
}

/// Production transport over Linear's Streamable HTTP MCP endpoint.
///
/// Each `send` call:
///   1. Fetches a fresh bearer token from the `TokenProvider`.
///   2. Posts a JSON-RPC envelope to the MCP endpoint.
///   3. Deserializes and classifies the response.
pub struct StreamableHttpTransport<P: TokenProvider> {
    config: StreamableHttpConfig,
    provider: Arc<P>,
    http: Client,
    next_id: AtomicU64,
}

impl<P: TokenProvider> StreamableHttpTransport<P> {
    pub fn new(config: StreamableHttpConfig, provider: Arc<P>, http: Client) -> Self {
        Self {
            config,
            provider,
            http,
            next_id: AtomicU64::new(1),
        }
    }

    /// Build with default config and a custom endpoint URL.
    pub fn with_endpoint(endpoint: Url, provider: Arc<P>, http: Client) -> Self {
        Self::new(
            StreamableHttpConfig {
                endpoint,
                ..Default::default()
            },
            provider,
            http,
        )
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Classify a non-2xx HTTP status into the appropriate `LinearMcpError`.
    fn classify_http_error(
        status: reqwest::StatusCode,
        body: String,
        headers: &reqwest::header::HeaderMap,
    ) -> LinearMcpError {
        match status.as_u16() {
            401 | 403 => LinearMcpError::Auth {
                reason: format!("HTTP {status}: {body}"),
            },
            429 => {
                let retry_after = headers
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                LinearMcpError::RateLimited {
                    retry_after_secs: retry_after,
                }
            }
            400 | 404 | 405 | 422 => LinearMcpError::InvalidRequest {
                reason: format!("HTTP {status}: {body}"),
            },
            s if s >= 500 => LinearMcpError::ServerError { status: s, body },
            _ => LinearMcpError::ServerError {
                status: status.as_u16(),
                body,
            },
        }
    }
}

impl<P: TokenProvider> std::fmt::Debug for StreamableHttpTransport<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamableHttpTransport")
            .field("endpoint", &self.config.endpoint)
            .field("request_timeout", &self.config.request_timeout)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl<P: TokenProvider> McpTransport for StreamableHttpTransport<P> {
    #[instrument(
        skip(self, request),
        fields(
            mcp.method  = %request.method,
            mcp.id      = request.id,
            http.url    = %self.config.endpoint,
        ),
        name = "mcp.transport.send"
    )]
    async fn send(&self, request: McpRequest) -> Result<McpResponse, LinearMcpError> {
        // 1. Resolve a fresh (or cached) bearer token.
        let token = self.provider.token().await?;

        debug!(mcp.method = %request.method, "Sending MCP request");

        // 2. POST the JSON-RPC envelope, with per-request timeout.
        let http_response = tokio::time::timeout(
            self.config.request_timeout,
            self.http
                .post(self.config.endpoint.clone())
                // Linear's Streamable HTTP transport uses these headers.
                .header("Authorization", format!("Bearer {}", token.expose_secret()))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json, text/event-stream")
                .json(&request)
                .send(),
        )
        .await
        .map_err(|_| LinearMcpError::Timeout {
            timeout_ms: self.config.request_timeout.as_millis() as u64,
        })?
        .map_err(LinearMcpError::Transport)?;

        let status = http_response.status();
        let headers = http_response.headers().clone();

        // 3. Classify non-2xx responses before attempting JSON decode.
        if !status.is_success() {
            let body = http_response.text().await.unwrap_or_default();
            warn!(
                http.status = status.as_u16(),
                http.body   = %body,
                "MCP transport received non-2xx response"
            );
            return Err(Self::classify_http_error(status, body, &headers));
        }

        // 4. Deserialize the JSON-RPC response envelope.
        let bytes = http_response
            .bytes()
            .await
            .map_err(LinearMcpError::Transport)?;

        let mcp_response: McpResponse =
            serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_slice(&bytes))
                .map_err(|e| LinearMcpError::Deserialize {
                    path: e.path().to_string(),
                    source: e.into_inner(),
                })?;

        debug!(mcp.id = mcp_response.id, "MCP response received");
        Ok(mcp_response)
    }
}
