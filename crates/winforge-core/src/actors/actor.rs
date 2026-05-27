use async_trait::async_trait;
use uuid::Uuid;

use super::context::ActorContext;

/// Unique identity for a spawned actor.
pub type ActorId = Uuid;

/// Core trait that all actors must implement.
///
/// Each actor runs in its own tokio task, communicates exclusively through
/// typed messages, and may optionally hook into lifecycle events.
#[async_trait]
pub trait Actor: Send + 'static {
    /// The message type this actor accepts.
    type Message: Send + 'static;

    /// Called once before the actor begins processing messages.
    async fn on_start(&mut self, _ctx: &ActorContext) {}

    /// Called for every message delivered to this actor.
    async fn receive(&mut self, msg: Self::Message, ctx: &ActorContext);

    /// Called once after the actor's mailbox is closed and drained.
    async fn on_stop(&mut self) {}
}
