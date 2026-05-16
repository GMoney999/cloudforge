// src/client/raw.rs

use crate::{
    client::retry::{with_retry, RetryConfig},
    error::LinearMcpError,
    transport::{McpRequest, McpTransport},
};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tracing::{info, instrument};

/// Discovered MCP tool metadata.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

/// The raw MCP client. Handles:
/// - JSON-RPC request sequencing (monotonic IDs)
/// - `tools/list` → `Vec<ToolInfo>`
/// - `tools/call`  → raw `serde_json::Value`
/// - Retry/backoff for transient failures
///
/// This is the protocol foundation. The typed `TypedLinearClient`
/// is built on top of this.
#[derive(Debug, Clone)]
pub struct RawMcpClient<T: McpTransport> {
    transport: Arc<T>,
    retry: RetryConfig,
    next_id: Arc<AtomicU64>,
}

impl<T: McpTransport> RawMcpClient<T> {
    pub fn new(transport: T, retry: RetryConfig) -> Self {
        Self {
            transport: Arc::new(transport),
            retry,
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn with_default_retry(transport: T) -> Self {
        Self::new(transport, RetryConfig::default())
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Discover all tools advertised by the Linear MCP server.
    #[instrument(skip(self), name = "mcp.list_tools")]
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>, LinearMcpError> {
        let transport = Arc::clone(&self.transport);
        let id = self.next_id();

        let result = with_retry(&self.retry, "tools/list", || {
            let t = Arc::clone(&transport);
            let req = McpRequest::new(id, "tools/list", json!({}));
            async move { t.send(req).await?.into_result() }
        })
        .await?;

        let tools: Vec<ToolInfo> = serde_path_to_error::deserialize(
            result.get("tools").unwrap_or(&result),
        )
        .map_err(|e| LinearMcpError::Deserialize {
            path: e.path().to_string(),
            source: e.into_inner(),
        })?;

        info!(tool_count = tools.len(), "Discovered MCP tools");
        Ok(tools)
    }

    /// Invoke a named MCP tool with an arbitrary JSON argument map.
    /// Returns the raw `content` value from the MCP response.
    #[instrument(
        skip(self, arguments),
        fields(mcp.tool = %tool_name),
        name = "mcp.call_tool"
    )]
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, LinearMcpError> {
        let transport = Arc::clone(&self.transport);
        let id = self.next_id();
        let tool_name_owned = tool_name.to_string();

        let result = with_retry(&self.retry, tool_name, || {
            let t = Arc::clone(&transport);
            let name = tool_name_owned.clone();
            let args = arguments.clone();
            let req = McpRequest::new(id, "tools/call", json!({ "name": name, "arguments": args }));
            async move { t.send(req).await?.into_result() }
        })
        .await?;

        // MCP tools/call returns { content: [...] }
        // Extract the first text content block for convenience.
        let content = result.get("content").cloned().unwrap_or(result);

        Ok(content)
    }
}
