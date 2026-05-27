//! WinForge Host Runtime
//!
//! Wraps the core runtime (actors, events, plugins, workflows) and exposes it
//! to external shells — primarily the WinUI 3 C# shell — over two named pipes:
//!
//! - `\\.\pipe\winforge-shell-cmd` — command/response (shell → host)
//! - `\\.\pipe\winforge-shell-evt` — push events    (host → shell)

pub mod bridge;
pub mod protocol;
pub mod runtime;

pub use bridge::run_bridge;
pub use runtime::HostRuntime;
