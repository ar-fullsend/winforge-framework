use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::step::StepDefinition;

/// Top-level structure of a `*.workflow.yaml` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,

    #[serde(default = "default_version")]
    pub version: String,

    pub description: Option<String>,

    /// Events or schedules that can start this workflow.
    #[serde(default)]
    pub triggers: Vec<Trigger>,

    /// Ordered list of steps (dependencies expressed via `depends_on`).
    pub steps: Vec<StepDefinition>,

    /// Workflow-level environment variables available to all steps.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Maximum wall-clock seconds before the workflow is forcibly cancelled.
    pub timeout_secs: Option<u64>,
}

fn default_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    /// Fired when a named event is published on the event bus.
    Event { topic: String },
    /// Fired on a cron-like schedule (future).
    Schedule { cron: String },
    /// Fired manually via the CLI or API.
    Manual,
}

impl WorkflowDefinition {
    /// Parse from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Load from a file on disk.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(Self::from_yaml(&contents)?)
    }
}
