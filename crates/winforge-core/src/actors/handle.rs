use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::{CoreError, CoreResult};

use super::actor::ActorId;

/// A typed, cloneable handle for sending messages to a running actor.
///
/// Dropping all handles causes the actor's mailbox to close, which triggers
/// `on_stop` and terminates the actor task.
#[derive(Debug)]
pub struct ActorHandle<M: Send + 'static> {
    id: ActorId,
    sender: mpsc::Sender<M>,
}

impl<M: Send + 'static> Clone for ActorHandle<M> {
    fn clone(&self) -> Self {
        Self { id: self.id, sender: self.sender.clone() }
    }
}

impl<M: Send + 'static> ActorHandle<M> {
    pub(crate) fn new(id: ActorId, sender: mpsc::Sender<M>) -> Self {
        Self { id, sender }
    }

    /// Returns the unique ID of the actor this handle points to.
    pub fn id(&self) -> ActorId {
        self.id
    }

    /// Sends a message, awaiting capacity in the mailbox.
    pub async fn send(&self, msg: M) -> CoreResult<()> {
        self.sender
            .send(msg)
            .await
            .map_err(|_| CoreError::ActorDead(self.id))
    }

    /// Sends a message without blocking, returning immediately if the mailbox is full.
    pub fn try_send(&self, msg: M) -> CoreResult<()> {
        match self.sender.try_send(msg) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(CoreError::MailboxFull),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(CoreError::ActorDead(self.id)),
        }
    }

    /// Returns `true` if the actor task is still running.
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }
}
