// src/linear/issues.rs

//! Issue-domain helpers for `TypedLinearClient`.
//!
//! All public functions are free functions that accept a `&RawMcpClient<T>`
//! so they can be unit-tested independently of the full client struct and
//! re-used in any future specialised client (e.g. an agent-facing adapter).

use crate::{
    client::RawMcpClient,
    error::LinearMcpError,
    transport::McpTransport,
    types::issue::{CreateIssueInput, Issue, IssueFilter, IssuePage, UpdateIssueInput},
};
use serde_json::{json, Value};
use tracing::instrument;

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Deserialise a raw `Value` into `T`, attaching field-path context on error.
fn parse<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, LinearMcpError> {
    serde_path_to_error::deserialize(value).map_err(|e| LinearMcpError::Deserialize {
        path: e.path().to_string(),
        source: e.into_inner(),
    })
}

/// Serialise `T` into a `Value`, mapping any error to `InvalidRequest`.
fn to_args<T: serde::Serialize>(input: &T) -> Result<Value, LinearMcpError> {
    serde_json::to_value(input).map_err(|e| LinearMcpError::InvalidRequest {
        reason: format!("failed to serialise request input: {e}"),
    })
}

// ── Public domain functions ───────────────────────────────────────────────────

/// Create a new Linear issue.
///
/// Maps to the `createIssue` MCP tool.
/// Returns the full `Issue` as confirmed by the Linear server.
#[instrument(
    skip(raw, input),
    fields(
        mcp.tool      = "createIssue",
        issue.title   = %input.title,
        issue.team_id = %input.team_id,
    ),
    name = "linear.issues.create"
)]
pub async fn create_issue<T: McpTransport>(
    raw: &RawMcpClient<T>,
    input: CreateIssueInput,
) -> Result<Issue, LinearMcpError> {
    let args = to_args(&input)?;
    let value = raw.call_tool("createIssue", args).await?;
    parse(value)
}

/// List issues with optional filters.
///
/// Maps to the `listIssues` MCP tool.
/// Returns a cursor-paginated `IssuePage`.
#[instrument(
    skip(raw, filter),
    fields(
        mcp.tool        = "listIssues",
        filter.team_id  = filter.team_id.as_ref().map(|id| id.to_string()).as_deref(),
        filter.priority = ?filter.priority,
        filter.first    = ?filter.first,
    ),
    name = "linear.issues.list"
)]
pub async fn list_issues<T: McpTransport>(
    raw: &RawMcpClient<T>,
    filter: IssueFilter,
) -> Result<IssuePage, LinearMcpError> {
    let args = to_args(&filter)?;
    let value = raw.call_tool("listIssues", args).await?;
    parse(value)
}

/// Fetch a single issue by its UUID or human-readable identifier (e.g. `"ENG-42"`).
///
/// Maps to the `getIssue` MCP tool.
#[instrument(
    skip(raw),
    fields(mcp.tool = "getIssue", issue.id = %issue_id),
    name = "linear.issues.get"
)]
pub async fn get_issue<T: McpTransport>(
    raw: &RawMcpClient<T>,
    issue_id: &str,
) -> Result<Issue, LinearMcpError> {
    let args = json!({ "id": issue_id });
    let value = raw.call_tool("getIssue", args).await?;
    parse(value)
}

/// Update fields on an existing issue.
///
/// Maps to the `updateIssue` MCP tool.
/// Only fields set to `Some(...)` in `UpdateIssueInput` are sent;
/// `None` fields are omitted from the JSON payload entirely.
#[instrument(
    skip(raw, input),
    fields(mcp.tool = "updateIssue", issue.id = %issue_id),
    name = "linear.issues.update"
)]
pub async fn update_issue<T: McpTransport>(
    raw: &RawMcpClient<T>,
    issue_id: &str,
    input: UpdateIssueInput,
) -> Result<Issue, LinearMcpError> {
    // Merge the `id` field into the serialised input map.
    let mut args = to_args(&input)?;
    args.as_object_mut()
        .expect("UpdateIssueInput always serialises to an object")
        .insert("id".into(), json!(issue_id));
    let value = raw.call_tool("updateIssue", args).await?;
    parse(value)
}

/// Delete an issue by ID.
///
/// Maps to the `deleteIssue` MCP tool.
/// Returns `true` if the issue was successfully deleted.
#[instrument(
    skip(raw),
    fields(mcp.tool = "deleteIssue", issue.id = %issue_id),
    name = "linear.issues.delete"
)]
pub async fn delete_issue<T: McpTransport>(
    raw: &RawMcpClient<T>,
    issue_id: &str,
) -> Result<bool, LinearMcpError> {
    let args = json!({ "id": issue_id });
    let value = raw.call_tool("deleteIssue", args).await?;
    // Linear returns `{ "success": true }` on deletion.
    let success = value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(success)
}

/// Search issues by a free-text query string.
///
/// Maps to the `searchIssues` MCP tool.
/// Returns a cursor-paginated `IssuePage`.
#[instrument(
    skip(raw),
    fields(mcp.tool = "searchIssues", query = %query),
    name = "linear.issues.search"
)]
pub async fn search_issues<T: McpTransport>(
    raw: &RawMcpClient<T>,
    query: &str,
    first: Option<u32>,
) -> Result<IssuePage, LinearMcpError> {
    let mut args = json!({ "query": query });
    if let Some(n) = first {
        args.as_object_mut()
            .unwrap()
            .insert("first".into(), json!(n));
    }
    let value = raw.call_tool("searchIssues", args).await?;
    parse(value)
}
