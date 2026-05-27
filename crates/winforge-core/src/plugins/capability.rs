use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

/// Fine-grained capability tokens that gate what a plugin may do.
///
/// Each capability is a `namespace:action` pair. The loader checks that a plugin
/// declares every capability it actually uses; the host enforces them at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // Filesystem
    FilesystemRead,
    FilesystemWrite,
    FilesystemDelete,

    // Network
    NetworkOutbound,
    NetworkInbound,
    NetworkListen,

    // Events
    EventsPublish,
    EventsSubscribe,

    // Processes
    ProcessSpawn,
    ProcessKill,

    // Registry (Windows)
    RegistryRead,
    RegistryWrite,

    // Services (Windows)
    ServiceQuery,
    ServiceControl,

    // Secrets
    SecretsRead,

    // UI
    UiNotify,
    UiWindow,

    // IPC
    IpcNamedPipe,
    IpcSharedMemory,

    /// Catch-all for custom capabilities declared by host extensions.
    Custom(String),
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilesystemRead => write!(f, "filesystem:read"),
            Self::FilesystemWrite => write!(f, "filesystem:write"),
            Self::FilesystemDelete => write!(f, "filesystem:delete"),
            Self::NetworkOutbound => write!(f, "network:outbound"),
            Self::NetworkInbound => write!(f, "network:inbound"),
            Self::NetworkListen => write!(f, "network:listen"),
            Self::EventsPublish => write!(f, "events:publish"),
            Self::EventsSubscribe => write!(f, "events:subscribe"),
            Self::ProcessSpawn => write!(f, "process:spawn"),
            Self::ProcessKill => write!(f, "process:kill"),
            Self::RegistryRead => write!(f, "registry:read"),
            Self::RegistryWrite => write!(f, "registry:write"),
            Self::ServiceQuery => write!(f, "service:query"),
            Self::ServiceControl => write!(f, "service:control"),
            Self::SecretsRead => write!(f, "secrets:read"),
            Self::UiNotify => write!(f, "ui:notify"),
            Self::UiWindow => write!(f, "ui:window"),
            Self::IpcNamedPipe => write!(f, "ipc:named_pipe"),
            Self::IpcSharedMemory => write!(f, "ipc:shared_memory"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

impl FromStr for Capability {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "filesystem:read" => Ok(Self::FilesystemRead),
            "filesystem:write" => Ok(Self::FilesystemWrite),
            "filesystem:delete" => Ok(Self::FilesystemDelete),
            "network:outbound" => Ok(Self::NetworkOutbound),
            "network:inbound" => Ok(Self::NetworkInbound),
            "network:listen" => Ok(Self::NetworkListen),
            "events:publish" => Ok(Self::EventsPublish),
            "events:subscribe" => Ok(Self::EventsSubscribe),
            "process:spawn" => Ok(Self::ProcessSpawn),
            "process:kill" => Ok(Self::ProcessKill),
            "registry:read" => Ok(Self::RegistryRead),
            "registry:write" => Ok(Self::RegistryWrite),
            "service:query" => Ok(Self::ServiceQuery),
            "service:control" => Ok(Self::ServiceControl),
            "secrets:read" => Ok(Self::SecretsRead),
            "ui:notify" => Ok(Self::UiNotify),
            "ui:window" => Ok(Self::UiWindow),
            "ipc:named_pipe" => Ok(Self::IpcNamedPipe),
            "ipc:shared_memory" => Ok(Self::IpcSharedMemory),
            other => {
                if let Some(rest) = other.strip_prefix("custom:") {
                    Ok(Self::Custom(rest.to_string()))
                } else {
                    Err(CoreError::CapabilityDenied(format!("unknown capability: {other}")))
                }
            }
        }
    }
}
