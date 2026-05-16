// tests/unit/transport_mock.rs

//! Unit tests for the raw MCP client using MockTransport.
//! No network, no credentials required.

use linear_mcp_client::{
    client::{RawMcpClient, RetryConfig},
    error::LinearMcpError,
    transport::MockTransport,
};
use serde_json::json;

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
}

#[tokio::test]
async fn list_tools_returns_parsed_tool_names() {
    setup_tracing();

    let mock = MockTransport::new();
    mock.enqueue_ok(
        "tools/list",
        json!({
            "tools": [
                { "name": "createIssue", "description": "Create a Linear issue" },
                { "name": "listIssues",  "description": "List Linear issues"   },
            ]
        }),
    );

    let client = RawMcpClient::with_default_retry(mock);
    let tools = client.list_tools().await.expect("list_tools failed");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "createIssue");
    assert_eq!(tools[1].name, "listIssues");
}

#[tokio::test]
async fn call_tool_returns_raw_content() {
    setup_tracing();

    let mock = MockTransport::new();
    mock.enqueue_ok(
        "tools/call",
        json!({
            "content": [{ "type": "text", "text": "{\"id\": \"abc-123\"}" }]
        }),
    );

    let client = RawMcpClient::with_default_retry(mock);
    let result = client
        .call_tool(
            "createIssue",
            json!({ "title": "Test", "teamId": "team-1" }),
        )
        .await
        .expect("call_tool failed");

    assert!(result.is_array());
}

#[tokio::test]
async fn retries_on_server_error_then_succeeds() {
    setup_tracing();

    let mock = MockTransport::new();

    // First attempt: 503
    mock.enqueue_err(
        "tools/call",
        LinearMcpError::ServerError {
            status: 503,
            body: "Service Unavailable".into(),
        },
    );
    // Second attempt: success
    mock.enqueue_ok(
        "tools/call",
        json!({ "content": [{ "type": "text", "text": "ok" }] }),
    );

    let retry = RetryConfig {
        max_retries: 2,
        base_delay: Duration::from_millis(1), // fast in tests
        max_delay: Duration::from_millis(10),
    };
    let client = RawMcpClient::new(mock, retry);

    let result = client
        .call_tool("listTeams", json!({}))
        .await
        .expect("should succeed after retry");

    assert!(result.is_array());
}

#[tokio::test]
async fn does_not_retry_auth_errors() {
    setup_tracing();

    let mock = MockTransport::new();
    mock.enqueue_err(
        "tools/call",
        LinearMcpError::Auth {
            reason: "Invalid token".into(),
        },
    );

    let retry = RetryConfig {
        max_retries: 3,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
    };
    let client = RawMcpClient::new(mock, retry);

    let err = client
        .call_tool("listTeams", json!({}))
        .await
        .expect_err("auth error should not be retried");

    assert!(matches!(err, LinearMcpError::Auth { .. }));
    // Only 1 entry was enqueued — if retry happened, mock would panic.
}

#[tokio::test]
async fn exhausts_retries_and_returns_last_error() {
    setup_tracing();

    let mock = MockTransport::new();
    // Enqueue one more error than max_retries to confirm exact attempt count.
    for _ in 0..=2 {
        mock.enqueue_err(
            "tools/call",
            LinearMcpError::ServerError {
                status: 500,
                body: "Internal Server Error".into(),
            },
        );
    }

    let retry = RetryConfig {
        max_retries: 2,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(5),
    };
    let client = RawMcpClient::new(mock, retry);

    let err = client
        .call_tool("listTeams", json!({}))
        .await
        .expect_err("should fail after exhausting retries");

    assert!(matches!(
        err,
        LinearMcpError::ServerError { status: 500, .. }
    ));
}
