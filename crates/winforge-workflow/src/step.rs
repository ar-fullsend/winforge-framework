use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Definition of a single workflow step as declared in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub id: String,

    pub name: Option<String>,

    #[serde(flatten)]
    pub kind: StepKind,

    /// IDs of steps that must complete before this step starts.
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Per-step environment overrides.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Number of retry attempts on failure (0 = no retry).
    #[serde(default)]
    pub retries: u32,

    /// Step-level timeout in seconds.
    pub timeout_secs: Option<u64>,

    /// If true, step failure does not abort the workflow.
    #[serde(default)]
    pub continue_on_error: bool,
}

/// The execution strategy for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepKind {
    /// Run an external command or script.
    Command {
        run: String,
        /// Working directory (relative to the workflow file or absolute).
        cwd: Option<String>,
    },

    /// Publish an event on the bus.
    Event {
        emit: EventEmit,
    },

    /// Wait until an event is received (with optional timeout).
    Await {
        topic: String,
    },

    /// Run nested steps in parallel; wait for all to complete.
    Parallel {
        steps: Vec<StepDefinition>,
    },

    /// Conditional branching.
    Decision {
        condition: String,
        if_true: Box<StepDefinition>,
        if_false: Option<Box<StepDefinition>>,
    },

    /// Loop over items.
    ForEach {
        items: String,
        step: Box<StepDefinition>,
    },

    /// Send a notification (toast / log / webhook).
    Notify {
        message: String,
        channel: Option<String>,
    },

    /// Compensating action for saga pattern.
    Compensate {
        run: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEmit {
    pub topic: String,
    pub payload: Option<serde_json::Value>,
}
