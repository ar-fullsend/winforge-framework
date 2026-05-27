use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Commands sent from the shell to the host (cmd pipe).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub id: String,
    #[serde(flatten)]
    pub command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum Command {
    Ping,
    GetStatus,
    ListPlugins,
    ListWorkflows {
        #[serde(default = "default_workflows_dir")]
        dir: String,
    },
    RunWorkflow {
        path: String,
        #[serde(default)]
        context: HashMap<String, Value>,
    },
}

fn default_workflows_dir() -> String {
    "workflows".to_string()
}

/// Responses sent from the host to the shell (cmd pipe, one per command).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub id: String,
    #[serde(flatten)]
    pub response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum Response {
    Pong,
    Status {
        uptime_secs: u64,
        plugin_count: usize,
        running_workflows: usize,
    },
    Plugins {
        list: Vec<PluginInfo>,
    },
    Workflows {
        list: Vec<WorkflowInfo>,
    },
    WorkflowStarted {
        workflow_id: String,
        name: String,
    },
    Error {
        message: String,
    },
}

/// Events pushed from the host to the shell (evt pipe, unsolicited).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    #[serde(flatten)]
    pub event: PushEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum PushEvent {
    WorkflowStepStarted { workflow_id: String, step_id: String },
    WorkflowStepCompleted { workflow_id: String, step_id: String, status: String },
    WorkflowCompleted { workflow_id: String, status: String },
    PluginLoaded { name: String, version: String },
    Log { level: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub path: String,
    pub step_count: usize,
}
