// tests/integration/smoke.rs

//! Optional live smoke tests against Linear's real MCP endpoint.
//! Skipped automatically when `LINEAR_API_KEY` is not set.
//! Run with:
//!   LINEAR_API_KEY=lin_api_xxx cargo test --test integration -- --nocapture

use linear_mcp_client::builder::LinearClientBuilder;
use linear_mcp_client::linear::LinearClient;

fn api_key() -> Option<String> {
    std::env::var("LINEAR_API_KEY").ok()
}

#[tokio::test]
async fn smoke_list_tools() {
    let Some(key) = api_key() else {
        eprintln!("Skipping smoke test — LINEAR_API_KEY not set");
        return;
    };

    let client = LinearClientBuilder::new()
        .bearer_token(key)
        .build()
        .expect("builder failed");

    // list_tools lives on the raw layer; access via the builder's
    // exposed raw client. Here we test the typed layer instead.
    let teams = client.list_teams().await.expect("list_teams failed");
    assert!(
        !teams.is_empty(),
        "Expected at least one team in the Linear workspace"
    );
    println!("Teams: {teams:#?}");
}

#[tokio::test]
async fn smoke_list_issues() {
    let Some(key) = api_key() else {
        eprintln!("Skipping smoke test — LINEAR_API_KEY not set");
        return;
    };

    let client = LinearClientBuilder::new()
        .bearer_token(key)
        .build()
        .expect("builder failed");

    let page = client
        .list_issues(Default::default())
        .await
        .expect("list_issues failed");

    println!("Issues ({} returned):", page.nodes.len());
    for issue in &page.nodes {
        println!("  {} — {}", issue.identifier, issue.title);
    }
}
