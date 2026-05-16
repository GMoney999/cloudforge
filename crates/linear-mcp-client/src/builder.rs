// src/builder.rs

use crate::{
    auth::{BearerTokenProvider, TokenProvider},
    client::{RawMcpClient, RetryConfig},
    error::LinearMcpError,
    linear::TypedLinearClient,
    transport::streamable_http::{StreamableHttpConfig, StreamableHttpTransport},
};
use reqwest::Client;
use std::{sync::Arc, time::Duration};
use url::Url;

const DEFAULT_ENDPOINT: &str = "https://mcp.linear.app/mcp";

/// Fluent builder for constructing a `TypedLinearClient`.
///
/// # Minimal example (bearer token)
/// ```rust,no_run
/// # use linear_mcp_client::builder::LinearClientBuilder;
/// # #[tokio::main] async fn main() -> anyhow::Result<()> {
/// let client = LinearClientBuilder::new()
///     .bearer_token("lin_api_xxxx")
///     .build()?;
/// # Ok(()) }
/// ```
///
/// # With custom endpoint + retry
/// ```rust,no_run
/// # use linear_mcp_client::builder::LinearClientBuilder;
/// # use std::time::Duration;
/// # #[tokio::main] async fn main() -> anyhow::Result<()> {
/// let client = LinearClientBuilder::new()
///     .bearer_token_from_env("LINEAR_API_KEY")?
///     .endpoint("https://mcp.linear.app/mcp")?
///     .request_timeout(Duration::from_secs(20))
///     .max_retries(5)
///     .build()?;
/// # Ok(()) }
/// ```
#[derive(Debug, Default)]
pub struct LinearClientBuilder {
    endpoint: Option<Url>,
    token_provider: Option<Arc<dyn TokenProvider>>,
    retry: RetryConfig,
    request_timeout: Option<Duration>,
    session_timeout: Option<Duration>,
}

impl LinearClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the MCP server endpoint URL.
    /// Defaults to `https://mcp.linear.app/mcp`.
    pub fn endpoint(mut self, url: impl AsRef<str>) -> Result<Self, LinearMcpError> {
        self.endpoint = Some(url.as_ref().parse()?);
        Ok(self)
    }

    /// Authenticate with a static bearer token.
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.token_provider = Some(Arc::new(BearerTokenProvider::new(token)));
        self
    }

    /// Authenticate with a bearer token read from an environment variable.
    pub fn bearer_token_from_env(mut self, var: &str) -> Result<Self, LinearMcpError> {
        self.token_provider = Some(Arc::new(BearerTokenProvider::from_env(var)?));
        Ok(self)
    }

    /// Inject any custom `TokenProvider` implementation (e.g. OAuth).
    pub fn token_provider<P: TokenProvider>(mut self, provider: P) -> Self {
        self.token_provider = Some(Arc::new(provider));
        self
    }

    /// Per-request HTTP timeout. Defaults to 30 seconds.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Maximum lifetime of an MCP session. Defaults to 1 hour.
    pub fn session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = Some(timeout);
        self
    }

    /// Override the default maximum retry count (default: 3).
    pub fn max_retries(mut self, n: u32) -> Self {
        self.retry.max_retries = n;
        self
    }

    /// Override the base backoff delay (default: 500ms).
    pub fn base_delay(mut self, d: Duration) -> Self {
        self.retry.base_delay = d;
        self
    }

    /// Finalise and construct the client.
    pub fn build(
        self,
    ) -> Result<TypedLinearClient<StreamableHttpTransport<Arc<dyn TokenProvider>>>, LinearMcpError>
    {
        let provider: Arc<dyn TokenProvider> =
            self.token_provider.ok_or_else(|| LinearMcpError::Auth {
                reason: "No token provider configured. \
                         Call .bearer_token() or .token_provider()."
                    .into(),
            })?;

        let endpoint = self
            .endpoint
            .unwrap_or_else(|| DEFAULT_ENDPOINT.parse().expect("hard-coded URL is valid"));

        let http = Client::builder()
            .use_rustls_tls()
            // reqwest connection-level timeout (separate from per-request).
            .connection_verbose(false)
            .build()
            .map_err(LinearMcpError::Transport)?;

        let transport_config = StreamableHttpConfig {
            endpoint,
            request_timeout: self
                .request_timeout
                .unwrap_or(StreamableHttpConfig::default().request_timeout),
            session_timeout: self
                .session_timeout
                .or(StreamableHttpConfig::default().session_timeout),
        };

        let transport = StreamableHttpTransport::new(transport_config, Arc::clone(&provider), http);

        let raw = RawMcpClient::new(transport, self.retry);
        Ok(TypedLinearClient::new(raw))
    }
}
