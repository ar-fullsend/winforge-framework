use std::sync::Arc;
use uuid::Uuid;

use crate::events::EventBus;

/// Passed to every actor lifecycle method, giving it access to the runtime.
#[derive(Clone)]
pub struct ActorContext {
    pub actor_id: Uuid,
    pub event_bus: Arc<EventBus>,
}

impl ActorContext {
    pub(crate) fn new(actor_id: Uuid, event_bus: Arc<EventBus>) -> Self {
        Self { actor_id, event_bus }
    }
}
