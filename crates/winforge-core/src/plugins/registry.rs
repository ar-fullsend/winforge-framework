use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{error, info, warn};

use crate::error::{CoreError, CoreResult};

use super::capability::Capability;
use super::manifest::PluginManifest;

/// The host-side API surface exposed to plugins.
pub struct PluginHost {
    pub granted_capabilities: HashSet<Capability>,
}

impl PluginHost {
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.granted_capabilities.contains(cap)
    }

    pub fn require_capability(&self, cap: &Capability) -> CoreResult<()> {
        if self.has_capability(cap) {
            Ok(())
        } else {
            Err(CoreError::CapabilityDenied(cap.to_string()))
        }
    }
}

/// Trait that every loaded plugin must implement.
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    async fn on_load(&mut self, host: &PluginHost) -> CoreResult<()>;
    async fn on_unload(&mut self) -> CoreResult<()>;

    /// Expose implementation-specific data (for host extension points).
    fn as_any(&self) -> &dyn Any;
}

/// Registration record kept by the registry.
struct PluginEntry {
    manifest: PluginManifest,
    plugin: Box<dyn Plugin>,
    path: PathBuf,
}

/// Central plugin registry.
///
/// Plugins are registered via [`PluginRegistry::register`] (for statically linked plugins)
/// or discovered from a plugin directory via [`PluginRegistry::discover`].
pub struct PluginRegistry {
    plugins: HashMap<String, PluginEntry>,
    /// Capabilities granted to all plugins by the host policy. A production host
    /// should narrow this per-plugin after reading the manifest.
    granted: HashSet<Capability>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new(), granted: HashSet::new() }
    }

    /// Grant a capability to all subsequently loaded plugins.
    pub fn grant(&mut self, cap: Capability) {
        self.granted.insert(cap);
    }

    /// Register a pre-constructed plugin along with its manifest.
    pub async fn register(
        &mut self,
        manifest: PluginManifest,
        mut plugin: Box<dyn Plugin>,
        path: PathBuf,
    ) -> CoreResult<()> {
        manifest.validate()?;
        let required = manifest.capabilities.required_capabilities()?;
        for cap in &required {
            if !self.granted.contains(cap) {
                return Err(CoreError::CapabilityDenied(cap.to_string()));
            }
        }

        let host = PluginHost { granted_capabilities: self.granted.clone() };
        plugin.on_load(&host).await?;

        let name = manifest.plugin.name.clone();
        info!(plugin = %name, version = %manifest.plugin.version, "plugin loaded");
        self.plugins.insert(name, PluginEntry { manifest, plugin, path });
        Ok(())
    }

    /// Walk `dir`, loading every sub-directory that contains a `plugin.toml`.
    pub async fn discover(&mut self, dir: &Path) -> CoreResult<usize> {
        let mut count = 0;
        let read = std::fs::read_dir(dir)?;
        for entry in read.flatten() {
            let plugin_dir = entry.path();
            if !plugin_dir.is_dir() {
                continue;
            }
            let manifest_path = plugin_dir.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }
            match PluginManifest::load(&plugin_dir) {
                Ok(manifest) => {
                    info!(dir = %plugin_dir.display(), "discovered plugin: {}", manifest.plugin.name);
                    // Dynamic loading is platform-specific; emit a warning for now
                    // until the platform-specific loader is wired up.
                    warn!(
                        plugin = %manifest.plugin.name,
                        "dynamic library loading not yet implemented; skipping entry_point"
                    );
                    count += 1;
                }
                Err(e) => {
                    error!(dir = %plugin_dir.display(), "failed to parse plugin.toml: {e}");
                }
            }
        }
        Ok(count)
    }

    /// Unload all plugins in reverse registration order.
    pub async fn shutdown(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names.into_iter().rev() {
            if let Some(mut entry) = self.plugins.remove(&name) {
                if let Err(e) = entry.plugin.on_unload().await {
                    error!(plugin = %name, "error during unload: {e}");
                }
                info!(plugin = %name, "plugin unloaded");
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|e| e.plugin.as_ref())
    }

    pub fn loaded_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
