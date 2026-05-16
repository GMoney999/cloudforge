use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Newtype wrapper for Linear entity IDs.
/// Prevents accidental mixing of e.g. TeamId and IssueId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LinearId(pub Uuid);

impl LinearId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for LinearId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Timestamp = DateTime<Utc>;

/// Cursor-based pagination info returned by list operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

/// A page of results with pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub nodes: Vec<T>,
    pub page_info: PageInfo,
}
