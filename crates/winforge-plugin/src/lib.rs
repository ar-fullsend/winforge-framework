//! WinForge Plugin SDK
//!
//! Provides the ergonomic surface for writing WinForge plugins. Import this crate
//! in your plugin crate instead of `winforge-core` directly.
//!
//! # Minimal plugin
//!
//! ```rust,no_run
//! use winforge_plugin::prelude::*;
//!
//! pub struct MyPlugin;
//!
//! #[async_trait]
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn version(&self) -> &str { "0.1.0" }
//!
//!     async fn on_load(&mut self, host: &PluginHost) -> CoreResult<()> {
//!         tracing::info!("my-plugin loaded");
//!         Ok(())
//!     }
//!
//!     async fn on_unload(&mut self) -> CoreResult<()> {
//!         Ok(())
//!     }
//!
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//! }
//! ```

pub use winforge_core::{
    Capability, CoreError, CoreResult, Event, EventBus, EventReceiver, Plugin, PluginHost,
    PluginManifest,
};
pub use async_trait::async_trait;

/// Convenient glob import for plugin authors.
pub mod prelude {
    pub use super::*;
    pub use std::any::Any;
    pub use tracing::{debug, error, info, warn};
}

/// Helper to build a `PluginManifest` in code rather than from TOML.
pub struct PluginManifestBuilder {
    name: String,
    version: String,
    description: Option<String>,
    authors: Vec<String>,
    entry_point: String,
    requires: Vec<String>,
    optional: Vec<String>,
    emits: Vec<String>,
    listens: Vec<String>,
}

impl PluginManifestBuilder {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            authors: vec![],
            entry_point: String::new(),
            requires: vec![],
            optional: vec![],
            emits: vec![],
            listens: vec![],
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }

    pub fn entry_point(mut self, ep: impl Into<String>) -> Self {
        self.entry_point = ep.into();
        self
    }

    pub fn requires(mut self, cap: Capability) -> Self {
        self.requires.push(cap.to_string());
        self
    }

    pub fn optional(mut self, cap: Capability) -> Self {
        self.optional.push(cap.to_string());
        self
    }

    pub fn emits(mut self, topic: impl Into<String>) -> Self {
        self.emits.push(topic.into());
        self
    }

    pub fn listens(mut self, topic: impl Into<String>) -> Self {
        self.listens.push(topic.into());
        self
    }

    pub fn build(self) -> PluginManifest {
        use winforge_core::plugins::{CapabilityDeclaration, EventDeclaration, PluginMeta};
        PluginManifest {
            plugin: PluginMeta {
                name: self.name,
                version: self.version,
                description: self.description,
                authors: if self.authors.is_empty() { None } else { Some(self.authors) },
                entry_point: self.entry_point,
                sha256: None,
            },
            capabilities: CapabilityDeclaration {
                requires: self.requires,
                optional: self.optional,
            },
            events: EventDeclaration {
                emits: self.emits,
                listens: self.listens,
            },
            dependencies: vec![],
        }
    }
}
