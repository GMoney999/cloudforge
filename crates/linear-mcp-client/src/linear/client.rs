// src/linear/client.rs

use super::LinearClient;
use crate::{
    client::RawMcpClient,
    error::LinearMcpError,
    transport::McpTransport,
    types::{
        Comment, CreateCommentInput, CreateIssueInput, Issue, IssueFilter, IssuePage,
        Project, Team, UpdateIssueInput,
    },
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

/// Converts a `serde_json::Value` returned by the raw MCP client into
/// a typed struct, attaching path context on failure.
fn parse_result<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, LinearMcpError> {
    serde_path_to_error::deserialize(value).map_err(|e| LinearMcpError::Deserialize {
        path: e.path().to_string(),
        source: e.into_inner(),
    })
}

/// Typed Linear client built on top of `RawMcpClient`.
/// Each method translates ergonomic Rust inputs into the correct
/// MCP tool name + JSON arguments, then parses the response.
#[derive(Debug, Clone)]
pub struct TypedLinearClient<T: McpTransport> {
    raw: Arc<RawMcpClient<T>>,
}

impl<T: McpTransport> TypedLinearClient<T> {
    pub fn new(raw: RawMcpClient<T>) -> Self {
        Self { raw: Arc::new(raw) }
    }
}

#[async_trait]
impl<T: McpTransport> LinearClient for TypedLinearClient<T> {
    // ── Issues ────────────────────────────────────────────────────────────────

    #[instrument(skip(self, input), name = "linear.create_issue")]
    async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue, LinearMcpError> {
        let args = serde_json::to_value(&input).map_err(|e| LinearMcpError::Deserialize {
            path: "<input>".into(),
            source: e,
        })?;
        let raw = self.raw.call_tool("createIssue", args).await?;
        parse_result(raw)
    }

    #[instrument(skip(self, filter), name = "linear.list_issues")]
    async fn list_issues(&self, filter: IssueFilter) -> Result<IssuePage, LinearMcpError> {
        let args = serde_json::to_value(&filter).map_err(|e| LinearMcpError::Deserialize {
            path: "<filter>".into(),
            source: e,
        })?;
        let raw = self.raw.call_tool("listIssues", args).await?;
        parse_result(raw)
    }

    #[instrument(skip(self), fields(issue.id = %issue_id), name = "linear.get_issue")]
    async fn get_issue(&self, issue_id: &str) -> Result<Issue, LinearMcpError> {
        let raw = self
            .raw
            .call_tool("getIssue", serde_json::json!({ "id": issue_id }))
            .await?;
        parse_result(raw)
    }

    #[instrument(skip(self, input), fields(issue.id = %issue_id), name = "linear.update_issue")]
    async fn update_issue(
        &self,
        issue_id: &str,
        input: UpdateIssueInput,
    ) -> Result<Issue, LinearMcpError> {
        let mut args = serde_json::to_value(&input).map_err(|e| LinearMcpError::Deserialize {
            path: "<input>".into(),
            source: e,
        })?;
        args["id"] = serde_json::json!(issue_id);
        let raw = self.raw.call_tool("updateIssue", args).await?;
        parse_result(raw)
    }

    // ── Comments ──────────────────────────────────────────────────────────────

    #[instrument(skip(self, input), name = "linear.add_comment")]
    async fn add_comment(&self, input: CreateCommentInput) -> Result<Comment, LinearMcpError> {
        let args = serde_json::to_value(&input).map_err(|e| LinearMcpError::Deserialize {
            path: "<input>".into(),
            source: e,
        })?;
        let raw = self.raw.call_tool("createComment", args).await?;
        parse_result(raw)
    }

    #[instrument(skip(self), fields(issue.id = %issue_id), name = "linear.list_comments")]
    async fn list_comments(&self, issue_id: &str) -> Result<Vec<Comment>, LinearMcpError> {
        let raw = self
            .raw
            .call_tool("listComments", serde_json::json!({ "issueId": issue_id }))
            .await?;
        parse_result(raw)
    }

    // ── Teams ─────────────────────────────────────────────────────────────────

    #[instrument(skip(self), name = "linear.list_teams")]
    async fn list_teams(&self) -> Result<Vec<Team>, LinearMcpError> {
        let raw = self
            .raw
            .call_tool("listTeams", serde_json::json!({}))
            .await?;
        parse_result(raw)
    }

    #[instrument(skip(self), fields(team.id = %team_id), name = "linear.get_team")]
    async fn get_team(&self, team_id: &str) -> Result<Team, LinearMcpError> {
        let raw = self
            .raw
            .call_tool("getTeam", serde_json::json!({ "id": team_id }))
            .await?;
        parse_result(raw)
    }

    // ── Projects ──────────────────────────────────────────────────────────────

    #[instrument(skip(self), name = "linear.list_projects")]
    async fn list_projects(&self, team_id: Option<&str>) -> Result<Vec<Project>, LinearMcpError> {
        let args = match team_id {
            Some(id) => serde_json::json!({ "teamId": id }),
            None => serde_json::json!({}),
        };
        let raw = self.raw.call_tool("listProjects", args).await?;
        parse_result(raw)
    }

    #[instrument(skip(self), fields(project.id = %project_id), name = "linear.get_project")]
    async fn get_project(&self, project_id: &str) -> Result<Project, LinearMcpError> {
        let raw = self
            .raw
            .call_tool("getProject", serde_json::json!({ "id": project_id }))
            .await?;
        parse_result(raw)
    }
}
