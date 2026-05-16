// src/types/comment.rs

use super::common::{LinearId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: LinearId,
    pub body: String,
    pub issue_id: LinearId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentInput {
    pub issue_id: LinearId,
    pub body: String,
}

/// All fields optional — only populated fields are sent to the server.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentInput {
    pub body: Option<String>,
}
