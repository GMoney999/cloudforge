use thiserror::Error;

/// The canonical error type for all Linear MCP client operations.
///
/// Auth errors and client-side request errors are non-retryable.
/// Transport and server-side errors (5xx, timeouts) are retryable.
#[derive(Debug, Error)]
pub enum LinearMcpError {
    // ── Transport ────────────────────────────────────────────────────────────
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("MCP protocol error: {message} (code: {code:?})")]
    Protocol { code: Option<i32>, message: String },

    // ── Auth ─────────────────────────────────────────────────────────────────
    #[error("Authentication failed: {reason}")]
    Auth { reason: String },

    #[error("Token refresh failed: {source}")]
    TokenRefresh {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ── Client-side (non-retryable) ───────────────────────────────────────────
    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },

    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    // ── Server-side (retryable) ───────────────────────────────────────────────
    #[error("Rate limited — retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },

    #[error("Server error {status}: {body}")]
    ServerError { status: u16, body: String },

    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    // ── Serialization ─────────────────────────────────────────────────────────
    #[error("Deserialization error at path `{path}`: {source}")]
    Deserialize {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    // ── Catch-all ─────────────────────────────────────────────────────────────
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl LinearMcpError {
    /// Returns `true` if this error class should trigger a retry.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Transport(_)
                | Self::RateLimited { .. }
                | Self::ServerError { status, .. } if *status >= 500
                | Self::Timeout { .. }
        )
    }

    /// Returns `true` if the error is definitively a client/auth mistake
    /// and retrying would be futile.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Auth { .. }
                | Self::InvalidRequest { .. }
                | Self::ToolNotFound { .. }
                | Self::UrlParse(_)
        )
    }
}
