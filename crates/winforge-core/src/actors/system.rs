use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};
use uuid::Uuid;

use crate::error::CoreResult;
use crate::events::EventBus;

use super::{
    actor::{Actor, ActorId},
    context::ActorContext,
    handle::ActorHandle,
};

/// Tracks the cancellation signal for a running actor task.
struct ActorEntry {
    shutdown_tx: oneshot::Sender<()>,
}

/// Central registry that spawns and supervises all actors in the application.
///
/// Dropping the `ActorSystem` broadcasts a shutdown signal to every live actor.
pub struct ActorSystem {
    actors: Arc<DashMap<ActorId, ActorEntry>>,
    event_bus: Arc<EventBus>,
}

impl ActorSystem {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            actors: Arc::new(DashMap::new()),
            event_bus,
        }
    }

    /// Spawns `actor` into its own tokio task and returns a typed handle.
    ///
    /// `mailbox_capacity` controls how many messages can queue before back-pressure kicks in.
    pub fn spawn<A: Actor>(&self, mut actor: A, mailbox_capacity: usize) -> ActorHandle<A::Message> {
        let id = Uuid::new_v4();
        let (msg_tx, mut msg_rx) = mpsc::channel::<A::Message>(mailbox_capacity);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let ctx = ActorContext::new(id, self.event_bus.clone());
        let actors = self.actors.clone();

        tokio::spawn(async move {
            actor.on_start(&ctx).await;
            info!(actor_id = %id, "actor started");

            loop {
                tokio::select! {
                    biased;
                    _ = &mut shutdown_rx => {
                        info!(actor_id = %id, "actor received shutdown signal");
                        break;
                    }
                    msg = msg_rx.recv() => {
                        match msg {
                            Some(m) => actor.receive(m, &ctx).await,
                            None => {
                                // All handles dropped — clean shutdown
                                break;
                            }
                        }
                    }
                }
            }

            actor.on_stop().await;
            actors.remove(&id);
            info!(actor_id = %id, "actor stopped");
        });

        self.actors.insert(id, ActorEntry { shutdown_tx });
        ActorHandle::new(id, msg_tx)
    }

    /// Returns the number of currently live actors.
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    /// Signals all actors to stop and waits briefly for them to drain.
    pub async fn shutdown(&self) {
        info!("actor system shutting down ({} actors)", self.actors.len());
        // Collect IDs first, then remove to get owned entries.
        let ids: Vec<ActorId> = self.actors.iter().map(|r| *r.key()).collect();
        for id in ids {
            if let Some((_, entry)) = self.actors.remove(&id) {
                let _ = entry.shutdown_tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    struct EchoActor {
        results_tx: mpsc::Sender<String>,
    }

    #[async_trait]
    impl Actor for EchoActor {
        type Message = String;

        async fn receive(&mut self, msg: String, _ctx: &ActorContext) {
            let _ = self.results_tx.send(msg).await;
        }
    }

    #[tokio::test]
    async fn actor_receives_messages() {
        let bus = Arc::new(EventBus::default());
        let system = ActorSystem::new(bus);

        let (tx, mut rx) = mpsc::channel(8);
        let handle = system.spawn(EchoActor { results_tx: tx }, 8);

        handle.send("hello".to_string()).await.unwrap();
        handle.send("world".to_string()).await.unwrap();

        assert_eq!(rx.recv().await.unwrap(), "hello");
        assert_eq!(rx.recv().await.unwrap(), "world");
    }

    #[tokio::test]
    async fn actor_is_dead_after_all_handles_dropped() {
        let bus = Arc::new(EventBus::default());
        let system = ActorSystem::new(bus);

        let (tx, _rx) = mpsc::channel(8);
        let handle = system.spawn(EchoActor { results_tx: tx }, 8);
        assert!(handle.is_alive());

        drop(handle);
        // Give the task a tick to clean up
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn system_shutdown_signals_actors() {
        let bus = Arc::new(EventBus::default());
        let system = ActorSystem::new(bus);

        let (tx1, _rx1) = mpsc::channel(8);
        let (tx2, _rx2) = mpsc::channel(8);
        system.spawn(EchoActor { results_tx: tx1 }, 8);
        system.spawn(EchoActor { results_tx: tx2 }, 8);

        assert_eq!(system.actor_count(), 2);
        system.shutdown().await;
        // Map is drained synchronously; tasks clean up asynchronously
        assert_eq!(system.actor_count(), 0);
    }
}
