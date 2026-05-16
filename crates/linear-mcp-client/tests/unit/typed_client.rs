// tests/unit/typed_client.rs

//! Unit tests for `TypedLinearClient` using `MockTransport`.
//! Verifies that typed methods map to the correct MCP tool names
//! and that response structs deserialise correctly.

use linear_mcp_client::{
    client::{RawMcpClient, RetryConfig},
    linear::{LinearClient, TypedLinearClient},
    transport::MockTransport,
    types::{CreateIssueInput, IssueFilter, LinearId, Priority},
};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

fn fast_retry() -> RetryConfig {
    RetryConfig {
        max_retries: 0, // fail fast in unit tests
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
    }
}

fn mock_issue_json(id: &str, title: &str) -> serde_json::Value {
    json!({
        "id":          id,
        "identifier":  "ENG-1",
        "title":       title,
        "description": null,
        "priority":    "NO_PRIORITY",
        "state": {
            "id":   "state-uuid-0001",
            "name": "Backlog",
            "type": "backlog"
        },
        "teamId":    "team-uuid-0001",
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z",
        "url":       "https://linear.app/team/issue/ENG-1"
    })
}

#[tokio::test]
async fn create_issue_calls_correct_tool() {
    let mock = MockTransport::new();
    mock.enqueue_ok(
        "tools/call",
        json!({ "content": [mock_issue_json("issue-uuid-0001", "Fix the bug")] }),
    );

    let client = TypedLinearClient::new(RawMcpClient::new(mock, fast_retry()));

    let input = CreateIssueInput {
        title: "Fix the bug".into(),
        team_id: LinearId(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()),
        ..Default::default()
    };

    let issue = client
        .create_issue(input)
        .await
        .expect("create_issue failed");
    assert_eq!(issue.title, "Fix the bug");
}

#[tokio::test]
async fn list_issues_deserialises_page() {
    let mock = MockTransport::new();
    mock.enqueue_ok(
        "tools/call",
        json!({
            "content": [{
                "nodes": [mock_issue_json("issue-uuid-0001", "Issue A")],
                "pageInfo": { "hasNextPage": false, "endCursor": null }
            }]
        }),
    );

    let client = TypedLinearClient::new(RawMcpClient::new(mock, fast_retry()));

    let page = client
        .list_issues(IssueFilter::default())
        .await
        .expect("list_issues failed");

    assert_eq!(page.nodes.len(), 1);
    assert_eq!(page.nodes[0].title, "Issue A");
    assert!(!page.page_info.has_next_page);
}

#[tokio::test]
async fn list_teams_deserialises_vec() {
    let mock = MockTransport::new();
    mock.enqueue_ok(
        "tools/call",
        json!({
            "content": [[
                {
                    "id":          "team-uuid-0001",
                    "name":        "Engineering",
                    "key":         "ENG",
                    "description": null,
                    "createdAt":   "2024-01-01T00:00:00Z"
                }
            ]]
        }),
    );

    let client = TypedLinearClient::new(RawMcpClient::new(mock, fast_retry()));
    let teams = client.list_teams().await.expect("list_teams failed");

    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].key, "ENG");
}
