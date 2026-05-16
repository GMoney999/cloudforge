// src/linear/mod.rs

pub mod client;
pub mod comments;
pub mod issues;
pub mod projects;
pub mod teams;

pub use client::TypedLinearClient;

use crate::{
    error::LinearMcpError,
    types::{
        Comment, CreateCommentInput, CreateIssueInput, Issue, IssueFilter, IssuePage,
        Project, Team, UpdateIssueInput,
    },
};
use async_trait::async_trait;

/// The public contract for the typed Linear client.
///
/// Back this with `TypedLinearClient` in production and a mock
/// in tests. Design principle: every method maps 1-to-1 with
/// a Linear MCP tool call.
#[async_trait]
pub trait LinearClient: Send + Sync + 'static {
    // ── Issues ────────────────────────────────────────────────────────────────
    async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue, LinearMcpError>;

    async fn list_issues(&self, filter: IssueFilter) -> Result<IssuePage, LinearMcpError>;

    async fn get_issue(&self, issue_id: &str) -> Result<Issue, LinearMcpError>;

    async fn update_issue(
        &self,
        issue_id: &str,
        input: UpdateIssueInput,
    ) -> Result<Issue, LinearMcpError>;

    // ── Comments ──────────────────────────────────────────────────────────────
    async fn add_comment(&self, input: CreateCommentInput) -> Result<Comment, LinearMcpError>;

    async fn list_comments(&self, issue_id: &str) -> Result<Vec<Comment>, LinearMcpError>;

    // ── Teams ─────────────────────────────────────────────────────────────────
    async fn list_teams(&self) -> Result<Vec<Team>, LinearMcpError>;

    async fn get_team(&self, team_id: &str) -> Result<Team, LinearMcpError>;

    // ── Projects ──────────────────────────────────────────────────────────────
    async fn list_projects(&self, team_id: Option<&str>) -> Result<Vec<Project>, LinearMcpError>;

    async fn get_project(&self, project_id: &str) -> Result<Project, LinearMcpError>;
}
