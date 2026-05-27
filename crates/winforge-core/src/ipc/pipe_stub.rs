//! Named-pipe stubs for non-Windows platforms.
//!
//! The types exist so the crate compiles everywhere; all methods return an
//! explicit `CoreError::Ipc` so callers get a clear error instead of a
//! compile-time failure when targeting non-Windows hosts.

use crate::error::{CoreError, CoreResult};
use super::IpcMessage;

pub struct IpcServer {
    #[allow(dead_code)]
    name: String,
}

impl IpcServer {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub async fn accept(&self) -> CoreResult<IpcConnection> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }

    pub fn bind(&self) -> CoreResult<PendingServer> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }
}

pub struct PendingServer;

impl PendingServer {
    pub async fn accept(self) -> CoreResult<IpcConnection> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }
}

pub struct IpcClient {
    #[allow(dead_code)]
    name: String,
}

impl IpcClient {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub async fn connect(&self) -> CoreResult<IpcConnection> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }
}

pub struct IpcConnection;

impl IpcConnection {
    pub async fn send(&mut self, _msg: &IpcMessage) -> CoreResult<()> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }

    pub async fn recv(&mut self) -> CoreResult<IpcMessage> {
        Err(CoreError::Ipc(
            "Named pipe IPC is only supported on Windows".into(),
        ))
    }
}
