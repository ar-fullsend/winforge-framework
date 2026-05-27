mod capability;
mod manifest;
mod registry;

pub use capability::Capability;
pub use manifest::{CapabilityDeclaration, EventDeclaration, PluginManifest, PluginMeta};
pub use registry::{Plugin, PluginHost, PluginRegistry};
