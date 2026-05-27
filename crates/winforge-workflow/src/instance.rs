use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub status: StepStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Runtime state of a single workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub context: HashMap<String, serde_json::Value>,
    pub steps: HashMap<String, StepResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl WorkflowInstance {
    pub fn new(id: Uuid, workflow_name: String, context: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id,
            workflow_name,
            status: WorkflowStatus::Pending,
            context,
            steps: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }
}
