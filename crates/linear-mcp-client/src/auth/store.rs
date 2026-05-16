use crate::error::LinearMcpError;
use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};

/// Abstraction over where tokens are persisted.
/// Implement this to back tokens with a keychain, Vault, DB, etc.
#[async_trait]
pub trait TokenStore: Send + Sync + 'static {
    async fn load(&self, key: &str) -> Result<Option<Secret<String>>, LinearMcpError>;
    async fn save(&self, key: &str, value: Secret<String>) -> Result<(), LinearMcpError>;
    async fn delete(&self, key: &str) -> Result<(), LinearMcpError>;
}

/// Reads tokens from environment variables. Suitable for CI and Docker.
#[derive(Debug, Clone, Default)]
pub struct EnvTokenStore;

#[async_trait]
impl TokenStore for EnvTokenStore {
    async fn load(&self, key: &str) -> Result<Option<Secret<String>>, LinearMcpError> {
        Ok(std::env::var(key).ok().map(Secret::new))
    }

    async fn save(&self, _key: &str, _value: Secret<String>) -> Result<(), LinearMcpError> {
        // Env vars are read-only at runtime; this is a no-op in this impl.
        Ok(())
    }

    async fn delete(&self, _key: &str) -> Result<(), LinearMcpError> {
        Ok(())
    }
}

/// In-memory token store backed by a `tokio::sync::RwLock`.
/// Suitable for tests and short-lived processes.
#[derive(Debug, Default)]
pub struct InMemoryTokenStore {
    inner: tokio::sync::RwLock<std::collections::HashMap<String, String>>,
}

#[async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn load(&self, key: &str) -> Result<Option<Secret<String>>, LinearMcpError> {
        let guard = self.inner.read().await;
        Ok(guard.get(key).cloned().map(Secret::new))
    }

    async fn save(&self, key: &str, value: Secret<String>) -> Result<(), LinearMcpError> {
        let mut guard = self.inner.write().await;
        guard.insert(key.to_string(), value.expose_secret().clone());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), LinearMcpError> {
        let mut guard = self.inner.write().await;
        guard.remove(key);
        Ok(())
    }
}
