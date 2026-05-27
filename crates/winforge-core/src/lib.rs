//! WinForge Core Runtime
//!
//! Provides the foundational building blocks for WinForge applications:
//! - **Actor system** — lightweight, tokio-backed actors with typed mailboxes
//! - **Event bus** — in-process pub/sub with per-type broadcast channels
//! - **Plugin system** — manifest-driven, capability-gated plugin registry
//! - **IPC** — named-pipe transport layer (Windows; stubs on other platforms)

pub mod actors;
pub mod error;
pub mod events;
pub mod ipc;
pub mod plugins;

// Re-export the most commonly used types at the crate root.
pub use actors::{Actor, ActorContext, ActorHandle, ActorSystem};
pub use error::{CoreError, CoreResult};
pub use events::{Event, EventBus, EventReceiver};
pub use plugins::{Capability, Plugin, PluginHost, PluginManifest, PluginRegistry};
pub use plugins::{load_plugin, verify_hash};
pub use ipc::{IpcClient, IpcConnection, IpcMessage, IpcServer};
