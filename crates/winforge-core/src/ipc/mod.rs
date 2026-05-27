pub mod framing;

#[cfg(target_os = "windows")]
#[path = "pipe_windows.rs"]
mod pipe_impl;

#[cfg(not(target_os = "windows"))]
#[path = "pipe_stub.rs"]
mod pipe_impl;

pub use pipe_impl::{IpcClient, IpcConnection, IpcServer};

use serde::{Deserialize, Serialize};

/// A length-framed message transmitted over an IPC channel.
/// The wire format is: 4-byte LE u32 length, then that many bytes of UTF-8 JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: String,
    pub topic: String,
    pub payload: serde_json::Value,
}
