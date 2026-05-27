use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

use winforge_core::ipc::framing::{read_frame, write_frame};
use winforge_workflow::WorkflowEngine;

use crate::protocol::{
    Command, CommandEnvelope, EventEnvelope, PluginInfo, PushEvent, Response, ResponseEnvelope,
};
use crate::runtime::HostRuntime;

// ── Platform-specific bridge ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub use windows_bridge::run_bridge;

#[cfg(not(target_os = "windows"))]
pub async fn run_bridge(_rt: Arc<HostRuntime>) -> anyhow::Result<()> {
    anyhow::bail!("Named pipe shell bridge is only supported on Windows")
}

#[cfg(target_os = "windows")]
mod windows_bridge {
    use super::*;
    use tokio::net::windows::named_pipe::ServerOptions;

    const CMD_PIPE: &str = r"\\.\pipe\winforge-shell-cmd";
    const EVT_PIPE: &str = r"\\.\pipe\winforge-shell-evt";

    /// Start both pipe servers and handle one shell connection at a time.
    /// Returns when the host is shutting down.
    pub async fn run_bridge(rt: Arc<HostRuntime>) -> anyhow::Result<()> {
        info!("shell bridge listening on {} and {}", CMD_PIPE, EVT_PIPE);

        loop {
            // Create fresh server instances for each connection attempt.
            let cmd_server = ServerOptions::new()
                .first_pipe_instance(false)
                .create(CMD_PIPE)?;
            let evt_server = ServerOptions::new()
                .first_pipe_instance(false)
                .create(EVT_PIPE)?;

            info!("waiting for shell to connect...");

            // Wait for both shell ends to connect.
            tokio::try_join!(cmd_server.connect(), evt_server.connect())?;
            info!("shell connected");

            let rt2 = rt.clone();
            let mut evt_rx = rt.subscribe_events();

            // Drive cmd pipe (receive commands, send responses).
            let cmd_task = tokio::spawn(handle_cmd_pipe(cmd_server, rt2));

            // Drive evt pipe (forward push events to shell).
            let evt_task = tokio::spawn(async move {
                handle_evt_pipe(evt_server, &mut evt_rx).await
            });

            // Wait for either end to disconnect.
            tokio::select! {
                res = cmd_task => { if let Err(e) = res { warn!("cmd pipe task: {e}"); } }
                res = evt_task => { if let Err(e) = res { warn!("evt pipe task: {e}"); } }
            }

            info!("shell disconnected, ready for next connection");
        }
    }

    async fn handle_cmd_pipe(
        mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
        rt: Arc<HostRuntime>,
    ) {
        loop {
            let json = match read_frame(&mut pipe).await {
                Ok(s) => s,
                Err(e) => { info!("cmd pipe closed: {e}"); break; }
            };

            let response_json = match serde_json::from_str::<CommandEnvelope>(&json) {
                Ok(env) => {
                    let response = dispatch_command(env.command, &rt).await;
                    let env_out = ResponseEnvelope { id: env.id, response };
                    serde_json::to_string(&env_out)
                        .unwrap_or_else(|e| format!(r#"{{"id":"err","kind":"Error","message":"{e}"}}"#))
                }
                Err(e) => {
                    error!("failed to parse command: {e}\nraw: {json}");
                    format!(r#"{{"id":"err","kind":"Error","message":"parse error: {e}"}}"#)
                }
            };

            if let Err(e) = write_frame(&mut pipe, &response_json).await {
                info!("cmd pipe write error: {e}");
                break;
            }
        }
    }

    async fn handle_evt_pipe(
        mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
        rx: &mut broadcast::Receiver<EventEnvelope>,
    ) {
        loop {
            match rx.recv().await {
                Ok(env) => {
                    let json = match serde_json::to_string(&env) {
                        Ok(s) => s,
                        Err(e) => { error!("serialize event: {e}"); continue; }
                    };
                    if let Err(e) = write_frame(&mut pipe, &json).await {
                        info!("evt pipe write error: {e}");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("evt pipe lagged, skipped {n} events");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}

// ── Command dispatch (platform-independent) ──────────────────────────────────

async fn dispatch_command(cmd: Command, rt: &Arc<HostRuntime>) -> Response {
    match cmd {
        Command::Ping => Response::Pong,

        Command::GetStatus => {
            let plugin_count = rt.plugins.lock().await.len();
            Response::Status {
                uptime_secs: rt.uptime_secs(),
                plugin_count,
                running_workflows: 0,
            }
        }

        Command::ListPlugins => {
            let registry = rt.plugins.lock().await;
            let list = registry
                .loaded_plugins()
                .into_iter()
                .filter_map(|name| {
                    // Plugin trait only exposes name/version; pull description from manifest
                    // via a downcast — here we emit what we have.
                    Some(PluginInfo {
                        name: name.to_string(),
                        version: "unknown".to_string(),
                        description: None,
                        capabilities: vec![],
                    })
                })
                .collect();
            Response::Plugins { list }
        }

        Command::ListWorkflows { dir } => {
            let list = rt.list_workflows(&dir);
            Response::Workflows { list }
        }

        Command::RunWorkflow { path, context } => {
            let def = match winforge_workflow::WorkflowDefinition::load(
                std::path::Path::new(&path),
            ) {
                Ok(d) => d,
                Err(e) => return Response::Error { message: e.to_string() },
            };

            let workflow_id = Uuid::new_v4().to_string();
            let wf_name = def.name.clone();
            let rt2 = rt.clone();
            let wf_id2 = workflow_id.clone();

            tokio::spawn(async move {
                let engine = WorkflowEngine::new(rt2.event_bus.clone());

                rt2.push(PushEvent::Log {
                    level: "info".into(),
                    message: format!("workflow '{}' starting", def.name),
                });

                match engine.run(&def, context).await {
                    Ok(instance) => {
                        for (step_id, result) in &instance.steps {
                            rt2.push(PushEvent::WorkflowStepCompleted {
                                workflow_id: wf_id2.clone(),
                                step_id: step_id.clone(),
                                status: format!("{:?}", result.status).to_lowercase(),
                            });
                        }
                        rt2.push(PushEvent::WorkflowCompleted {
                            workflow_id: wf_id2.clone(),
                            status: format!("{:?}", instance.status).to_lowercase(),
                        });
                    }
                    Err(e) => {
                        rt2.push(PushEvent::Log {
                            level: "error".into(),
                            message: format!("workflow failed: {e}"),
                        });
                        rt2.push(PushEvent::WorkflowCompleted {
                            workflow_id: wf_id2,
                            status: "failed".into(),
                        });
                    }
                }
            });

            Response::WorkflowStarted { workflow_id, name: wf_name }
        }
    }
}
