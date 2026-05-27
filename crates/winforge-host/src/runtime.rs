use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, Mutex};
use tracing::info;
use uuid::Uuid;

use winforge_core::{ActorSystem, EventBus, PluginRegistry};

use crate::protocol::{EventEnvelope, PushEvent, WorkflowInfo};

/// Shared runtime state, wrapped in Arc so both pipe bridges can access it.
pub struct HostRuntime {
    pub event_bus: Arc<EventBus>,
    pub actor_system: ActorSystem,
    pub plugins: Arc<Mutex<PluginRegistry>>,
    pub started_at: Instant,
    /// Channel for pushing events to all connected shells.
    pub event_tx: broadcast::Sender<EventEnvelope>,
}

impl HostRuntime {
    pub fn new() -> Self {
        let event_bus = Arc::new(EventBus::default());
        let actor_system = ActorSystem::new(event_bus.clone());
        let plugins = Arc::new(Mutex::new(PluginRegistry::new()));
        let (event_tx, _) = broadcast::channel(256);

        Self { event_bus, actor_system, plugins, started_at: Instant::now(), event_tx }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope> {
        self.event_tx.subscribe()
    }

    /// Push a `PushEvent` to all connected shells.
    pub fn push(&self, event: PushEvent) {
        let envelope = EventEnvelope { id: Uuid::new_v4().to_string(), event };
        // ignore if no subscribers
        let _ = self.event_tx.send(envelope);
    }

    /// Discover and list workflow YAML files in `dir`.
    pub fn list_workflows(&self, dir: &str) -> Vec<WorkflowInfo> {
        let path = Path::new(dir);
        if !path.exists() {
            return vec![];
        }
        std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "yaml" || ext == "yml")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let p = e.path();
                match winforge_workflow::WorkflowDefinition::load(&p) {
                    Ok(def) => Some(WorkflowInfo {
                        name: def.name,
                        version: def.version,
                        description: def.description,
                        path: p.to_string_lossy().into_owned(),
                        step_count: def.steps.len(),
                    }),
                    Err(_) => None,
                }
            })
            .collect()
    }

    pub async fn shutdown(self) {
        info!("host runtime shutting down");
        self.plugins.lock().await.shutdown().await;
        self.actor_system.shutdown().await;
    }
}
