//! Named-pipe IPC for Windows using `tokio::net::windows::named_pipe`.
//!
//! Wire format: every message is prefixed with a 4-byte little-endian u32
//! giving the byte length of the JSON payload that follows.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions};
use tracing::{debug, info};

use crate::error::{CoreError, CoreResult};

use super::IpcMessage;

// ERROR_PIPE_BUSY — all pipe instances are busy; retry.
const ERROR_PIPE_BUSY: i32 = 231;

fn pipe_path(name: &str) -> String {
    format!(r"\\.\pipe\winforge-{name}")
}

async fn send_msg(writer: &mut (impl AsyncWriteExt + Unpin), msg: &IpcMessage) -> CoreResult<()> {
    let bytes = serde_json::to_vec(msg)?;
    let len = bytes.len() as u32;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(&bytes).await?;
    Ok(())
}

async fn recv_msg(reader: &mut (impl AsyncReadExt + Unpin)) -> CoreResult<IpcMessage> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len == 0 || len > 64 * 1024 * 1024 {
        return Err(CoreError::Ipc(format!("invalid frame length: {len}")));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

// ── IpcServer ────────────────────────────────────────────────────────────────

/// Named-pipe server endpoint.
///
/// Call [`IpcServer::accept`] in a loop to handle sequential connections.
/// Each call creates a fresh pipe instance so the next client can connect
/// while the previous connection is still being processed in a spawned task.
pub struct IpcServer {
    pipe_name: String,
}

impl IpcServer {
    pub fn new(name: &str) -> Self {
        Self { pipe_name: pipe_path(name) }
    }

    /// Block until one client connects, then return the connected session.
    ///
    /// After this returns the *next* call to `accept` immediately creates a
    /// new server instance so the pipe name stays available.
    pub async fn accept(&self) -> CoreResult<IpcConnection> {
        let server = ServerOptions::new()
            .first_pipe_instance(false)  // allow re-use of the pipe name
            .create(&self.pipe_name)?;

        info!(pipe = %self.pipe_name, "waiting for IPC client");
        server.connect().await?;
        info!(pipe = %self.pipe_name, "IPC client connected");
        Ok(IpcConnection::server(server))
    }

    /// Create the first pipe instance and mark it as the owner.
    /// Use this when starting the server before any clients exist.
    pub fn bind(&self) -> CoreResult<PendingServer> {
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&self.pipe_name)?;
        Ok(PendingServer { inner: server, pipe_name: self.pipe_name.clone() })
    }
}

/// A created-but-not-yet-connected named pipe server instance.
pub struct PendingServer {
    inner: NamedPipeServer,
    pipe_name: String,
}

impl PendingServer {
    pub async fn accept(self) -> CoreResult<IpcConnection> {
        info!(pipe = %self.pipe_name, "waiting for IPC client");
        self.inner.connect().await?;
        info!(pipe = %self.pipe_name, "IPC client connected");
        Ok(IpcConnection::server(self.inner))
    }
}

// ── IpcClient ────────────────────────────────────────────────────────────────

/// Named-pipe client endpoint.
pub struct IpcClient {
    pipe_name: String,
}

impl IpcClient {
    pub fn new(name: &str) -> Self {
        Self { pipe_name: pipe_path(name) }
    }

    /// Connect to the server, retrying if the pipe is busy.
    pub async fn connect(&self) -> CoreResult<IpcConnection> {
        let retry_interval = Duration::from_millis(50);
        let max_wait = Duration::from_secs(10);
        let mut elapsed = Duration::ZERO;

        loop {
            match ClientOptions::new().open(&self.pipe_name) {
                Ok(client) => {
                    info!(pipe = %self.pipe_name, "IPC connected to server");
                    return Ok(IpcConnection::client(client));
                }
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    if elapsed >= max_wait {
                        return Err(CoreError::Ipc(format!(
                            "pipe '{}' still busy after {max_wait:?}",
                            self.pipe_name
                        )));
                    }
                    tokio::time::sleep(retry_interval).await;
                    elapsed += retry_interval;
                }
                Err(e) => return Err(CoreError::Io(e)),
            }
        }
    }
}

// ── IpcConnection ─────────────────────────────────────────────────────────────

/// A live bidirectional named-pipe session.
pub struct IpcConnection {
    inner: ConnectionInner,
}

enum ConnectionInner {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

impl IpcConnection {
    fn server(s: NamedPipeServer) -> Self {
        Self { inner: ConnectionInner::Server(s) }
    }
    fn client(c: NamedPipeClient) -> Self {
        Self { inner: ConnectionInner::Client(c) }
    }

    /// Send a message to the peer.
    pub async fn send(&mut self, msg: &IpcMessage) -> CoreResult<()> {
        match &mut self.inner {
            ConnectionInner::Server(s) => send_msg(s, msg).await,
            ConnectionInner::Client(c) => send_msg(c, msg).await,
        }
    }

    /// Receive the next message from the peer.
    pub async fn recv(&mut self) -> CoreResult<IpcMessage> {
        match &mut self.inner {
            ConnectionInner::Server(s) => recv_msg(s).await,
            ConnectionInner::Client(c) => recv_msg(c).await,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn roundtrip_single_message() {
        let pipe_name = format!("test-{}", Uuid::new_v4().as_simple());
        let server = IpcServer::new(&pipe_name);
        let client = IpcClient::new(&pipe_name);

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.expect("server accept");
            conn.recv().await.expect("server recv")
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client_conn = client.connect().await.expect("client connect");
        let sent = IpcMessage {
            id: "1".into(),
            topic: "test.event".into(),
            payload: serde_json::json!({ "hello": "world" }),
        };
        client_conn.send(&sent).await.expect("client send");

        let received = server_task.await.expect("server task");
        assert_eq!(received.id, "1");
        assert_eq!(received.topic, "test.event");
    }

    #[tokio::test]
    async fn server_client_bidirectional() {
        let pipe_name = format!("test-bidir-{}", Uuid::new_v4().as_simple());
        let server = IpcServer::new(&pipe_name);
        let client = IpcClient::new(&pipe_name);

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            let msg = conn.recv().await.unwrap();
            // Echo back with modified topic
            let reply = IpcMessage {
                id: msg.id.clone(),
                topic: format!("{}.reply", msg.topic),
                payload: msg.payload,
            };
            conn.send(&reply).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut conn = client.connect().await.unwrap();
        conn.send(&IpcMessage {
            id: "42".into(),
            topic: "ping".into(),
            payload: serde_json::Value::Null,
        })
        .await
        .unwrap();

        let reply = conn.recv().await.unwrap();
        assert_eq!(reply.topic, "ping.reply");
        assert_eq!(reply.id, "42");

        server_task.await.unwrap();
    }
}
