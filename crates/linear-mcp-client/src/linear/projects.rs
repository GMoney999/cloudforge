// src/linear/projects.rs

//! Project-domain helpers for `TypedLinearClient`.
//!
//! Projects in Linear group issues around a goal and can span multiple teams.
//! All functions accept an optional `team_id` to narrow the scope of queries.

use crate::{
    client::RawMcpClient,
    error::LinearMcpError,
    transport::McpTransport,
    types::project::{CreateProjectInput, Project, ProjectFilter, UpdateProjectInput},
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
        reason: format!("failed to serialise project input: {e}"),
    })
}

// ── Public domain functions ───────────────────────────────────────────────────

/// List projects, optionally scoped to a specific team.
///
/// Maps to the `listProjects` MCP tool.
/// Pass `filter.team_id = Some(id)` to narrow results to one team;
/// omit it to list projects across the whole workspace.
#[instrument(
    skip(raw, filter),
    fields(
        mcp.tool       = "listProjects",
        filter.team_id = filter.team_id.as_deref(),
        filter.state   = filter.state.as_deref(),
    ),
    name = "linear.projects.list"
)]
pub async fn list_projects<T: McpTransport>(
    raw: &RawMcpClient<T>,
    filter: ProjectFilter,
) -> Result<Vec<Project>, LinearMcpError> {
    let args = to_args(&filter)?;
    let value = raw.call_tool("listProjects", args).await?;
    parse(value)
}

/// Fetch a single project by its UUID.
///
/// Maps to the `getProject` MCP tool.
#[instrument(
    skip(raw),
    fields(mcp.tool = "getProject", project.id = %project_id),
    name = "linear.projects.get"
)]
pub async fn get_project<T: McpTransport>(
    raw: &RawMcpClient<T>,
    project_id: &str,
) -> Result<Project, LinearMcpError> {
    let args = json!({ "id": project_id });
    let value = raw.call_tool("getProject", args).await?;
    parse(value)
}

/// Create a new Linear project.
///
/// Maps to the `createProject` MCP tool.
/// Returns the newly created `Project` with its server-assigned ID.
#[instrument(
    skip(raw, input),
    fields(
        mcp.tool     = "createProject",
        project.name = %input.name,
    ),
    name = "linear.projects.create"
)]
pub async fn create_project<T: McpTransport>(
    raw: &RawMcpClient<T>,
    input: CreateProjectInput,
) -> Result<Project, LinearMcpError> {
    let args = to_args(&input)?;
    let value = raw.call_tool("createProject", args).await?;
    parse(value)
}

/// Update fields on an existing project.
///
/// Maps to the `updateProject` MCP tool.
/// Only `Some(...)` fields are serialised; `None` fields are omitted.
#[instrument(
    skip(raw, input),
    fields(mcp.tool = "updateProject", project.id = %project_id),
    name = "linear.projects.update"
)]
pub async fn update_project<T: McpTransport>(
    raw: &RawMcpClient<T>,
    project_id: &str,
    input: UpdateProjectInput,
) -> Result<Project, LinearMcpError> {
    let mut args = to_args(&input)?;
    args.as_object_mut()
        .expect("UpdateProjectInput always serialises to an object")
        .insert("id".into(), json!(project_id));
    let value = raw.call_tool("updateProject", args).await?;
    parse(value)
}
