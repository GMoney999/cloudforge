// src/transport/mod.rs

pub mod mock;
pub mod streamable_http;

pub use mock::MockTransport;
pub use streamable_http::StreamableHttpTransport;

use crate::error::LinearMcpError;
use async_trait::async_trait;
use serde_json::Value;

/// A single MCP JSON-RPC request envelope.
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

impl McpRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        }
    }
}

/// A raw MCP JSON-RPC response envelope.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpResponse {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<McpResponseError>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl McpResponse {
    /// Unwrap into the `result` value or convert the `error` field
    /// into a typed `LinearMcpError::Protocol`.
    pub fn into_result(self) -> Result<Value, LinearMcpError> {
        if let Some(err) = self.error {
            return Err(LinearMcpError::Protocol {
                code: Some(err.code),
                message: err.message,
            });
        }
        self.result.ok_or_else(|| LinearMcpError::Protocol {
            code: None,
            message: "MCP response contained neither `result` nor `error`".into(),
        })
    }
}

/// The core transport abstraction. Swap implementations for:
/// - `StreamableHttpTransport`  — production (Linear remote MCP)
/// - `MockTransport`            — unit / integration tests
/// - Future: `SseTransport`     — legacy Linear SSE endpoint
#[async_trait]
pub trait McpTransport: Send + Sync + 'static {
    /// Send a single JSON-RPC request and await its response.
    async fn send(&self, request: McpRequest) -> Result<McpResponse, LinearMcpError>;
}
