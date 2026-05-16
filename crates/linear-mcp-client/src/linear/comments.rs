// src/linear/comments.rs

//! Comment-domain helpers for `TypedLinearClient`.
//!
//! Each function maps 1-to-1 with a Linear MCP tool and is kept as
//! a plain async free function so it can be called from the typed
//! client, tested independently, or composed into agent workflows.

use crate::{
    client::RawMcpClient,
    error::LinearMcpError,
    transport::McpTransport,
    types::comment::{Comment, CreateCommentInput, UpdateCommentInput},
};
use serde_json::{json, Value};
use tracing::instrument;

// ── Internal helpers ──────────────────────────────────────────────────────────

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, LinearMcpError> {
    serde_path_to_error::deserialize(value).map_err(|e| LinearMcpError::Deserialize {
        path: e.path().to_string(),
        source: e.into_inner(),
    })
}

fn to_args<T: serde::Serialize>(input: &T) -> Result<Value, LinearMcpError> {
    serde_json::to_value(input).map_err(|e| LinearMcpError::InvalidRequest {
        reason: format!("failed to serialise comment input: {e}"),
    })
}

// ── Public domain functions ───────────────────────────────────────────────────

/// Add a comment to a Linear issue.
///
/// Maps to the `createComment` MCP tool.
/// Returns the newly created `Comment`.
#[instrument(
    skip(raw, input),
    fields(
        mcp.tool  = "createComment",
        issue.id  = %input.issue_id,
    ),
    name = "linear.comments.create"
)]
pub async fn add_comment<T: McpTransport>(
    raw: &RawMcpClient<T>,
    input: CreateCommentInput,
) -> Result<Comment, LinearMcpError> {
    let args = to_args(&input)?;
    let value = raw.call_tool("createComment", args).await?;
    parse(value)
}

/// List all comments on a given issue.
///
/// Maps to the `listComments` MCP tool.
/// Returns comments in ascending creation order.
#[instrument(
    skip(raw),
    fields(mcp.tool = "listComments", issue.id = %issue_id),
    name = "linear.comments.list"
)]
pub async fn list_comments<T: McpTransport>(
    raw: &RawMcpClient<T>,
    issue_id: &str,
) -> Result<Vec<Comment>, LinearMcpError> {
    let args = json!({ "issueId": issue_id });
    let value = raw.call_tool("listComments", args).await?;
    parse(value)
}

/// Fetch a single comment by its ID.
///
/// Maps to the `getComment` MCP tool.
#[instrument(
    skip(raw),
    fields(mcp.tool = "getComment", comment.id = %comment_id),
    name = "linear.comments.get"
)]
pub async fn get_comment<T: McpTransport>(
    raw: &RawMcpClient<T>,
    comment_id: &str,
) -> Result<Comment, LinearMcpError> {
    let args = json!({ "id": comment_id });
    let value = raw.call_tool("getComment", args).await?;
    parse(value)
}

/// Edit the body of an existing comment.
///
/// Maps to the `updateComment` MCP tool.
/// Returns the updated `Comment`.
#[instrument(
    skip(raw, input),
    fields(mcp.tool = "updateComment", comment.id = %comment_id),
    name = "linear.comments.update"
)]
pub async fn update_comment<T: McpTransport>(
    raw: &RawMcpClient<T>,
    comment_id: &str,
    input: UpdateCommentInput,
) -> Result<Comment, LinearMcpError> {
    let mut args = to_args(&input)?;
    args.as_object_mut()
        .expect("UpdateCommentInput always serialises to an object")
        .insert("id".into(), json!(comment_id));
    let value = raw.call_tool("updateComment", args).await?;
    parse(value)
}

/// Delete a comment by ID.
///
/// Maps to the `deleteComment` MCP tool.
/// Returns `true` if deletion was acknowledged by the server.
#[instrument(
    skip(raw),
    fields(mcp.tool = "deleteComment", comment.id = %comment_id),
    name = "linear.comments.delete"
)]
pub async fn delete_comment<T: McpTransport>(
    raw: &RawMcpClient<T>,
    comment_id: &str,
) -> Result<bool, LinearMcpError> {
    let args = json!({ "id": comment_id });
    let value = raw.call_tool("deleteComment", args).await?;
    let success = value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(success)
}
