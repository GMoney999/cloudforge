// src/linear/teams.rs

//! Team-domain helpers for `TypedLinearClient`.
//!
//! Teams are the top-level organisational unit in Linear.
//! Most other resources (issues, projects, cycles) are scoped to a team.

use crate::{
    client::RawMcpClient,
    error::LinearMcpError,
    transport::McpTransport,
    types::team::{Team, TeamMember},
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

// ── Public domain functions ───────────────────────────────────────────────────

/// List all teams the authenticated user belongs to.
///
/// Maps to the `listTeams` MCP tool.
/// Returns teams in the order Linear surfaces them (alphabetical by default).
#[instrument(
    skip(raw),
    fields(mcp.tool = "listTeams"),
    name = "linear.teams.list"
)]
pub async fn list_teams<T: McpTransport>(
    raw: &RawMcpClient<T>,
) -> Result<Vec<Team>, LinearMcpError> {
    let value = raw.call_tool("listTeams", json!({})).await?;
    parse(value)
}

/// Fetch a single team by its UUID.
///
/// Maps to the `getTeam` MCP tool.
#[instrument(
    skip(raw),
    fields(mcp.tool = "getTeam", team.id = %team_id),
    name = "linear.teams.get"
)]
pub async fn get_team<T: McpTransport>(
    raw: &RawMcpClient<T>,
    team_id: &str,
) -> Result<Team, LinearMcpError> {
    let args = json!({ "id": team_id });
    let value = raw.call_tool("getTeam", args).await?;
    parse(value)
}

/// List members of a team.
///
/// Maps to the `listTeamMembers` MCP tool.
/// Useful for resolving assignee IDs before creating or updating issues.
#[instrument(
    skip(raw),
    fields(mcp.tool = "listTeamMembers", team.id = %team_id),
    name = "linear.teams.list_members"
)]
pub async fn list_team_members<T: McpTransport>(
    raw: &RawMcpClient<T>,
    team_id: &str,
) -> Result<Vec<TeamMember>, LinearMcpError> {
    let args = json!({ "teamId": team_id });
    let value = raw.call_tool("listTeamMembers", args).await?;
    parse(value)
}
