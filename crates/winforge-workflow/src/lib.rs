//! WinForge Workflow Engine
//!
//! Parse and execute YAML-defined workflows. Workflows are composed of typed steps
//! (command, event, parallel, decision, for-each, notify) with explicit dependency
//! graphs. Steps with satisfied dependencies run concurrently.

pub mod definition;
pub mod engine;
pub mod error;
pub mod instance;
pub mod step;

pub use definition::WorkflowDefinition;
pub use engine::WorkflowEngine;
pub use error::{WorkflowError, WorkflowResult};
pub use instance::{StepResult, StepStatus, WorkflowInstance, WorkflowStatus};

use winforge_core::events::Event;

/// A generic event used by workflow steps to publish and await arbitrary bus messages.
#[derive(Debug, Clone)]
pub struct DynamicEvent {
    pub topic: String,
    pub payload: serde_json::Value,
}

impl Event for DynamicEvent {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use winforge_core::EventBus;

    fn simple_workflow_yaml() -> &'static str {
        r#"
name: test-workflow
version: "1.0"
steps:
  - id: step-a
    type: command
    run: "echo hello"
  - id: step-b
    type: notify
    message: "done"
    depends_on: [step-a]
"#
    }

    #[test]
    fn workflow_yaml_parses() {
        let def = WorkflowDefinition::from_yaml(simple_workflow_yaml()).unwrap();
        assert_eq!(def.name, "test-workflow");
        assert_eq!(def.steps.len(), 2);
    }

    #[tokio::test]
    async fn workflow_runs_to_completion() {
        let def = WorkflowDefinition::from_yaml(simple_workflow_yaml()).unwrap();
        let bus = Arc::new(EventBus::default());
        let engine = WorkflowEngine::new(bus);
        let instance = engine.run(&def, HashMap::new()).await.unwrap();
        assert_eq!(instance.status, WorkflowStatus::Completed);
        assert_eq!(instance.steps.len(), 2);
    }

    #[test]
    fn parallel_workflow_yaml_parses() {
        let yaml = r#"
name: parallel-test
version: "1.0"
steps:
  - id: fan-out
    type: parallel
    steps:
      - id: task-1
        type: notify
        message: "task 1"
      - id: task-2
        type: notify
        message: "task 2"
"#;
        let def = WorkflowDefinition::from_yaml(yaml).unwrap();
        assert_eq!(def.steps.len(), 1);
    }

    #[tokio::test]
    async fn cyclic_dependency_returns_error() {
        let yaml = r#"
name: cyclic
version: "1.0"
steps:
  - id: a
    type: notify
    message: "a"
    depends_on: [b]
  - id: b
    type: notify
    message: "b"
    depends_on: [a]
"#;
        let def = WorkflowDefinition::from_yaml(yaml).unwrap();
        let bus = Arc::new(EventBus::default());
        let engine = WorkflowEngine::new(bus);
        let result = engine.run(&def, HashMap::new()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WorkflowError::CyclicDependency));
    }
}
