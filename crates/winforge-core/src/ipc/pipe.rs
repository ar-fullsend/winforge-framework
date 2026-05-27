use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

/// A framed message sent over an IPC channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: String,
    pub topic: String,
    pub payload: serde_json::Value,
}

// ── Common placeholder for a connected session ────────────────────────────────

pub struct IpcConnection;

// ── Windows ───────────────────────────────────────────────────────────────────

/// Named-pipe IPC server (Windows only).
pub struct IpcServer {
    #[allow(dead_code)]
    name: String,
}

impl IpcServer {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub async fn accept(&self) -> CoreResult<IpcConnection> {
        #[cfg(target_os = "windows")]
        {
            // Real implementation uses windows-rs CreateNamedPipeW + ConnectNamedPipe.
            Err(CoreError::Ipc("Windows named pipe server: not yet implemented".into()))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(CoreError::Ipc("Named pipe IPC is only available on Windows".into()))
        }
    }
}

/// Named-pipe IPC client (Windows only).
pub struct IpcClient {
    #[allow(dead_code)]
    name: String,
}

impl IpcClient {
    pub fn connect(name: &str) -> CoreResult<Self> {
        #[cfg(target_os = "windows")]
        {
            Ok(Self { name: name.to_string() })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = name;
            Err(CoreError::Ipc("Named pipe IPC is only available on Windows".into()))
        }
    }

    pub async fn send(&self, _msg: &IpcMessage) -> CoreResult<()> {
        #[cfg(target_os = "windows")]
        {
            Err(CoreError::Ipc("Windows named pipe client: not yet implemented".into()))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(CoreError::Ipc("Named pipe IPC is only available on Windows".into()))
        }
    }
}
