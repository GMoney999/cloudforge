use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum ReportingError {
    #[error("Reporting failed: {0}")]
    SinkError(String),
    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgentResult {
    Success {
        summary: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        artifacts: Option<Value>,
    },
    Failure {
        error_message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        failed_step: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<Value>,
    },
    Partial {
        completed_tasks: Vec<String>,
        blocked_task: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportingPayload {
    pub namespace: String,
    pub agent_identity: String,
    pub result: AgentResult,
}

#[async_trait]
pub trait ReportingSink: Send + Sync {
    async fn report(&self, payload: ReportingPayload) -> Result<(), ReportingError>;
}


pub struct BrainReportingSink {
    client: reqwest::Client,
    endpoint_url: String,
}

impl BrainReportingSink {
    pub fn new(endpoint_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint_url,
        }
    }
}

#[async_trait]
impl ReportingSink for BrainReportingSink {
    async fn report(&self, payload: ReportingPayload) -> Result<(), ReportingError> {
        let response = self
            .client
            .post(&self.endpoint_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ReportingError::SinkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ReportingError::SinkError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        Ok(())
    }
}
