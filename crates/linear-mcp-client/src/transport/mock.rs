// src/transport/mock.rs

//! Mock transport for unit tests. Callers register expected
//! (method, params) → response pairs; the mock panics on unexpected calls.

use super::{McpRequest, McpResponse, McpTransport};
use crate::error::LinearMcpError;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::VecDeque;
use tokio::sync::Mutex;

/// A canned response entry stored in the mock queue.
#[derive(Debug)]
pub struct MockEntry {
    pub method: String,
    pub response: Result<Value, LinearMcpError>,
}

/// First-in-first-out mock transport.
///
/// # Example
/// ```rust
/// use linear_mcp_client::transport::MockTransport;
///
/// let transport = MockTransport::new();
/// transport.enqueue_ok("tools/list", serde_json::json!({ "tools": [] }));
/// ```
#[derive(Debug, Default)]
pub struct MockTransport {
    queue: Mutex<VecDeque<MockEntry>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a successful response for the given method.
    pub fn enqueue_ok(&self, method: impl Into<String>, result: Value) {
        // MockTransport is typically created once and used in a single-threaded
        // test setup, so blocking here is acceptable.
        let mut q = self.queue.blocking_lock();
        q.push_back(MockEntry {
            method: method.into(),
            response: Ok(result),
        });
    }

    /// Enqueue an error response for the given method.
    pub fn enqueue_err(&self, method: impl Into<String>, err: LinearMcpError) {
        let mut q = self.queue.blocking_lock();
        q.push_back(MockEntry {
            method: method.into(),
            response: Err(err),
        });
    }

    /// Assert all enqueued responses have been consumed (call at end of test).
    pub fn assert_exhausted(&self) {
        let q = self.queue.blocking_lock();
        assert!(
            q.is_empty(),
            "MockTransport: {} unconsumed response(s) remain",
            q.len()
        );
    }
}

#[async_trait]
impl McpTransport for MockTransport {
    async fn send(&self, request: McpRequest) -> Result<McpResponse, LinearMcpError> {
        let mut q = self.queue.lock().await;
        let entry = q.pop_front().unwrap_or_else(|| {
            panic!(
                "MockTransport: unexpected call to method `{}` — queue is empty",
                request.method
            )
        });

        assert_eq!(
            entry.method, request.method,
            "MockTransport: expected method `{}`, got `{}`",
            entry.method, request.method
        );

        match entry.response {
            Ok(result) => Ok(McpResponse {
                id: request.id,
                result: Some(result),
                error: None,
            }),
            Err(e) => Err(e),
        }
    }
}
