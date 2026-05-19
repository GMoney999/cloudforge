# Agent Reporting

The `agent-reporting` crate provides common structures and traits for reporting the status and results of AI agents in CloudForge.

## Usage

### `AgentResult`

The `AgentResult` enum represents the outcome of an agent's execution. It provides variants for successful completion, failures, and partial results.

```rust
use agent_reporting::AgentResult;
use serde_json::json;

// Successful execution with a summary and optional metadata/artifacts
let success = AgentResult::Success {
    summary: "Processed 50 items successfully".to_string(),
    metadata: Some(json!({ "items_processed": 50 })),
    artifacts: None,
};

// Failed execution with an error message and context
let failure = AgentResult::Failure {
    error_message: "Failed to connect to database".to_string(),
    failed_step: Some("database_connection".to_string()),
    context: Some(json!({ "db_host": "localhost" })),
};

// Partially completed task
let partial = AgentResult::Partial {
    completed_tasks: vec!["download_data".to_string()],
    blocked_task: "process_data".to_string(),
    reason: "Missing required fields in dataset".to_string(),
};
```

### `BrainReportingSink`

The `BrainReportingSink` implements the `ReportingSink` trait, allowing you to send `ReportingPayload`s to an HTTP endpoint.

```rust
use agent_reporting::{AgentResult, BrainReportingSink, ReportingPayload, ReportingSink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sink = BrainReportingSink::new("https://api.cloudforge.com/v1/report".to_string());
    
    let payload = ReportingPayload {
        namespace: "data-pipeline".to_string(),
        agent_identity: "agent-123".to_string(),
        result: AgentResult::Success {
            summary: "Job finished".to_string(),
            metadata: None,
            artifacts: None,
        },
    };

    // Report the result asynchronously
    sink.report(payload).await?;
    
    Ok(())
}
```

## Integration with Weavegraph Workflows

(Requires the optional `weavegraph` feature)

When integrating `agent-reporting` into Weavegraph workflows, agents act as individual nodes within a larger directed acyclic graph (DAG). The `BrainReportingSink` serves as the primary bridge between the executing agent and the Weavegraph orchestration engine.

1. **State Transitions**: When an agent completes its step, it submits an `AgentResult`. 
   - `AgentResult::Success` triggers the Weavegraph engine to transition to the next node(s) in the workflow.
   - `AgentResult::Failure` can trigger defined fallback paths, retries, or escalate alerts to operators.
   - `AgentResult::Partial` can signal human-in-the-loop (HITL) intervention or pause the workflow until unblocked.

2. **Data Passing**: The `metadata` and `artifacts` fields within `AgentResult::Success` are captured by the `BrainReportingSink` and written back to the Weavegraph context. This allows subsequent downstream agents in the workflow to consume the outputs of the current agent.

To enable Weavegraph specific functionality, add it to your `Cargo.toml`:
```toml
[dependencies]
agent-reporting = { version = "0.1.0", features = ["weavegraph"] }
```
