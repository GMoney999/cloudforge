use super::common::{LinearId, Page, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    NoPriority = 0,
    Urgent = 1,
    High = 2,
    Medium = 3,
    Low = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Self::NoPriority
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueState {
    pub id: LinearId,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String, // "backlog" | "started" | "completed" | "cancelled"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: LinearId,
    pub identifier: String, // e.g. "ENG-42"
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub state: IssueState,
    pub team_id: LinearId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub url: String,
}

/// Input for creating a new issue.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueInput {
    pub title: String,
    pub team_id: LinearId,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub state_id: Option<LinearId>,
    pub assignee_id: Option<LinearId>,
    pub project_id: Option<LinearId>,
    pub label_ids: Option<Vec<LinearId>>,
}

/// Input for updating an existing issue (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub state_id: Option<LinearId>,
    pub assignee_id: Option<LinearId>,
}

/// Filter parameters for listing issues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilter {
    pub team_id: Option<LinearId>,
    pub assignee_id: Option<LinearId>,
    pub state_types: Option<Vec<String>>,
    pub priority: Option<Priority>,
    pub first: Option<u32>,
    pub after: Option<String>, // cursor
}

pub type IssuePage = Page<Issue>;
