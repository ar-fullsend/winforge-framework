mod capability;
pub mod loader;
mod manifest;
mod registry;

pub use capability::Capability;
pub use loader::{load_plugin, verify_hash};
pub use manifest::{CapabilityDeclaration, EventDeclaration, PluginManifest, PluginMeta};
pub use registry::{Plugin, PluginHost, PluginRegistry};
