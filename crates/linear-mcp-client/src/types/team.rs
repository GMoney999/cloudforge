// src/types/team.rs

use super::common::{LinearId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: LinearId,
    pub name: String,
    pub key: String,
    pub description: Option<String>,
    pub created_at: Timestamp,
}

/// A member of a team, returned by `listTeamMembers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: LinearId,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
}
