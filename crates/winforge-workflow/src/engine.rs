use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use dashmap::DashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use winforge_core::events::EventBus;

use crate::error::WorkflowError;
use crate::instance::{StepResult, StepStatus, WorkflowInstance, WorkflowStatus};
use crate::step::{StepDefinition, StepKind};
use crate::{WorkflowDefinition, WorkflowResult};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Drives workflow execution.
pub struct WorkflowEngine {
    event_bus: Arc<EventBus>,
    instances: Arc<DashMap<Uuid, WorkflowInstance>>,
}

impl WorkflowEngine {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            instances: Arc::new(DashMap::new()),
        }
    }

    /// Start a new workflow instance and drive it to completion (or failure).
    pub async fn run(
        &self,
        definition: &WorkflowDefinition,
        context: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<WorkflowInstance> {
        let id = Uuid::new_v4();
        let mut instance = WorkflowInstance::new(id, definition.name.clone(), context);

        info!(workflow = %definition.name, instance_id = %id, "workflow starting");
        instance.status = WorkflowStatus::Running;

        let result = self.execute_steps(definition, &mut instance).await;

        match result {
            Ok(()) => {
                instance.status = WorkflowStatus::Completed;
                instance.completed_at = Some(Utc::now());
                info!(workflow = %definition.name, instance_id = %id, "workflow completed");
            }
            Err(ref e) => {
                instance.status = WorkflowStatus::Failed;
                instance.completed_at = Some(Utc::now());
                instance.error = Some(e.to_string());
                error!(workflow = %definition.name, instance_id = %id, error = %e, "workflow failed");
            }
        }

        self.instances.insert(id, instance.clone());
        result.map(|_| instance)
    }

    async fn execute_steps(
        &self,
        definition: &WorkflowDefinition,
        instance: &mut WorkflowInstance,
    ) -> WorkflowResult<()> {
        let total = definition.steps.len();
        let mut completed: HashSet<String> = HashSet::new();

        while completed.len() < total {
            let ready: Vec<StepDefinition> = definition
                .steps
                .iter()
                .filter(|s| {
                    !completed.contains(&s.id)
                        && !instance.steps.contains_key(&s.id)
                        && s.depends_on.iter().all(|dep| completed.contains(dep))
                })
                .cloned()
                .collect();

            if ready.is_empty() && completed.len() < total {
                return Err(WorkflowError::CyclicDependency);
            }

            let mut handles = vec![];
            for step in ready {
                let step_id = step.id.clone();
                let bus = self.event_bus.clone();
                let ctx = instance.context.clone();
                let handle =
                    tokio::spawn(async move { execute_step(&step, &ctx, bus).await });
                handles.push((step_id, handle));
            }

            for (step_id, handle) in handles {
                let result = handle.await.map_err(|e| WorkflowError::StepFailed {
                    step_id: step_id.clone(),
                    reason: e.to_string(),
                })?;

                match result {
                    Ok(step_result) => {
                        instance.steps.insert(step_id.clone(), step_result);
                        completed.insert(step_id);
                    }
                    Err(e) => {
                        let continue_on_err = definition
                            .steps
                            .iter()
                            .find(|s| s.id == step_id)
                            .map(|s| s.continue_on_error)
                            .unwrap_or(false);

                        if continue_on_err {
                            warn!(step_id = %step_id, error = %e, "step failed (continue_on_error=true)");
                            instance.steps.insert(
                                step_id.clone(),
                                StepResult {
                                    status: StepStatus::Skipped,
                                    output: None,
                                    error: Some(e.to_string()),
                                    started_at: Utc::now(),
                                    completed_at: Some(Utc::now()),
                                },
                            );
                            completed.insert(step_id);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_instance(&self, id: Uuid) -> Option<WorkflowInstance> {
        self.instances.get(&id).map(|r| r.clone())
    }

    pub fn running_count(&self) -> usize {
        self.instances
            .iter()
            .filter(|e| e.status == WorkflowStatus::Running)
            .count()
    }
}

fn execute_step<'a>(
    step: &'a StepDefinition,
    context: &'a HashMap<String, serde_json::Value>,
    event_bus: Arc<EventBus>,
) -> BoxFuture<'a, WorkflowResult<StepResult>> {
    Box::pin(async move {
        let started_at = Utc::now();
        info!(step_id = %step.id, "step starting");

        let timeout = step
            .timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(3600));

        let result =
            tokio::time::timeout(timeout, run_step_kind(step, context, event_bus)).await;

        match result {
            Ok(Ok(output)) => Ok(StepResult {
                status: StepStatus::Succeeded,
                output,
                error: None,
                started_at,
                completed_at: Some(Utc::now()),
            }),
            Ok(Err(e)) => Err(WorkflowError::StepFailed {
                step_id: step.id.clone(),
                reason: e.to_string(),
            }),
            Err(_) => Err(WorkflowError::StepTimeout { step_id: step.id.clone() }),
        }
    })
}

fn run_step_kind<'a>(
    step: &'a StepDefinition,
    context: &'a HashMap<String, serde_json::Value>,
    event_bus: Arc<EventBus>,
) -> BoxFuture<'a, WorkflowResult<Option<serde_json::Value>>> {
    Box::pin(async move {
        match &step.kind {
            StepKind::Command { run, cwd } => {
                info!(step_id = %step.id, cmd = %run, "executing command");
                let mut cmd = if cfg!(target_os = "windows") {
                    let mut c = tokio::process::Command::new("cmd");
                    c.args(["/C", run]);
                    c
                } else {
                    let mut c = tokio::process::Command::new("sh");
                    c.args(["-c", run]);
                    c
                };

                if let Some(dir) = cwd {
                    cmd.current_dir(dir);
                }
                for (k, v) in &step.env {
                    cmd.env(k, v);
                }

                let output = cmd.output().await.map_err(|e| WorkflowError::StepFailed {
                    step_id: step.id.clone(),
                    reason: format!("failed to spawn process: {e}"),
                })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(WorkflowError::StepFailed {
                        step_id: step.id.clone(),
                        reason: format!("command exited with {}: {stderr}", output.status),
                    });
                }

                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                Ok(Some(serde_json::Value::String(stdout)))
            }

            StepKind::Event { emit } => {
                info!(step_id = %step.id, topic = %emit.topic, "emitting event");
                let payload = emit.payload.clone().unwrap_or(serde_json::Value::Null);
                event_bus.publish(crate::DynamicEvent {
                    topic: emit.topic.clone(),
                    payload,
                });
                Ok(None)
            }

            StepKind::Notify { message, channel } => {
                let ch = channel.as_deref().unwrap_or("log");
                info!(step_id = %step.id, channel = %ch, "notify: {message}");
                Ok(None)
            }

            StepKind::Await { topic } => {
                info!(step_id = %step.id, topic = %topic, "awaiting event");
                let mut rx = event_bus.subscribe::<crate::DynamicEvent>();
                loop {
                    let evt = rx.recv().await.map_err(|e| WorkflowError::StepFailed {
                        step_id: step.id.clone(),
                        reason: e.to_string(),
                    })?;
                    if evt.topic == *topic {
                        return Ok(Some(evt.payload));
                    }
                }
            }

            StepKind::Parallel { steps } => {
                info!(step_id = %step.id, count = steps.len(), "running parallel steps");
                let mut handles = vec![];
                for sub in steps {
                    let sub = sub.clone();
                    let bus = event_bus.clone();
                    let ctx = context.clone();
                    handles.push(tokio::spawn(async move {
                        execute_step(&sub, &ctx, bus).await
                    }));
                }
                for handle in handles {
                    handle.await.map_err(|e| WorkflowError::StepFailed {
                        step_id: step.id.clone(),
                        reason: e.to_string(),
                    })??;
                }
                Ok(None)
            }

            StepKind::ForEach { items, step: inner } => {
                let arr = context
                    .get(items.as_str())
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                for item in arr {
                    let mut ctx = context.clone();
                    ctx.insert("item".to_string(), item);
                    let inner = inner.clone();
                    let bus = event_bus.clone();
                    execute_step(&inner, &ctx, bus).await?;
                }
                Ok(None)
            }

            StepKind::Decision { condition, if_true, if_false } => {
                let is_true = context
                    .get(condition.as_str())
                    .map(|v| match v {
                        serde_json::Value::Bool(b) => *b,
                        serde_json::Value::Null => false,
                        serde_json::Value::Number(n) => {
                            n.as_f64().map(|f| f != 0.0).unwrap_or(false)
                        }
                        serde_json::Value::String(s) => !s.is_empty(),
                        _ => true,
                    })
                    .unwrap_or(false);

                if is_true {
                    execute_step(if_true, context, event_bus).await?;
                } else if let Some(else_step) = if_false {
                    execute_step(else_step, context, event_bus).await?;
                }
                Ok(None)
            }

            StepKind::Compensate { run } => {
                info!(step_id = %step.id, cmd = %run, "running compensation (stub)");
                Ok(None)
            }
        }
    })
}
