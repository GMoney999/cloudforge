// src/types/project.rs

use super::common::{LinearId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: LinearId,
    pub name: String,
    pub description: Option<String>,
    /// Linear project states: `"planned"` | `"started"` | `"paused"`
    /// | `"completed"` | `"cancelled"`
    pub state: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Input for creating a new project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub team_ids: Vec<LinearId>,
    pub description: Option<String>,
    pub state: Option<String>,
}

/// All fields optional — only populated fields are sent to the server.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
}

/// Filter parameters for listing projects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFilter {
    pub team_id: Option<String>,
    pub state: Option<String>,
    pub first: Option<u32>,
    pub after: Option<String>, // cursor
}
