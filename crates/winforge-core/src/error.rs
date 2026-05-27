use uuid::Uuid;

/// Canonical error type for the WinForge core runtime.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("actor {0} is no longer alive")]
    ActorDead(Uuid),

    #[error("actor mailbox is full")]
    MailboxFull,

    #[error("actor system is shutting down")]
    SystemShutdown,

    #[error("plugin error: {0}")]
    Plugin(String),

    #[error("plugin manifest invalid: {0}")]
    InvalidManifest(String),

    #[error("plugin capability denied: {0}")]
    CapabilityDenied(String),

    #[error("event bus closed")]
    BusClosed,

    #[error("workflow error: {0}")]
    Workflow(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
