pub mod bearer;
pub mod oauth;
pub mod store;

pub use bearer::BearerTokenProvider;
pub use oauth::{OAuthToken, OAuthTokenProvider};
pub use store::{EnvTokenStore, InMemoryTokenStore, TokenStore};

use crate::error::LinearMcpError;
use async_trait::async_trait;

/// Core trait: anything that can provide a bearer token string for a request.
/// Implementations must be `Send + Sync + 'static` for use behind `Arc`.
#[async_trait]
pub trait TokenProvider: Send + Sync + 'static {
    /// Returns a valid bearer token, refreshing if necessary.
    async fn token(&self) -> Result<secrecy::Secret<String>, LinearMcpError>;
}
