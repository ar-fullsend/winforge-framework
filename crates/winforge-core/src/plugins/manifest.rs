use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

use super::capability::Capability;

/// Parsed representation of a `plugin.toml` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,

    #[serde(default)]
    pub capabilities: CapabilityDeclaration,

    #[serde(default)]
    pub events: EventDeclaration,

    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub entry_point: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityDeclaration {
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

impl CapabilityDeclaration {
    pub fn required_capabilities(&self) -> CoreResult<HashSet<Capability>> {
        self.requires
            .iter()
            .map(|s| Capability::from_str(s))
            .collect()
    }

    pub fn optional_capabilities(&self) -> CoreResult<HashSet<Capability>> {
        self.optional
            .iter()
            .map(|s| Capability::from_str(s))
            .collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventDeclaration {
    #[serde(default)]
    pub emits: Vec<String>,
    #[serde(default)]
    pub listens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
}

impl PluginManifest {
    pub fn from_str(toml: &str) -> CoreResult<Self> {
        toml::from_str(toml).map_err(CoreError::Toml)
    }

    pub fn load(plugin_dir: &Path) -> CoreResult<Self> {
        let path = plugin_dir.join("plugin.toml");
        let contents = std::fs::read_to_string(&path)?;
        Self::from_str(&contents)
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.plugin.name.is_empty() {
            return Err(CoreError::InvalidManifest("plugin.name must not be empty".into()));
        }
        if self.plugin.version.is_empty() {
            return Err(CoreError::InvalidManifest("plugin.version must not be empty".into()));
        }
        self.capabilities.required_capabilities()?;
        self.capabilities.optional_capabilities()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TOML: &str = r#"
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "A test plugin"
entry_point = "my_plugin.dll"

[capabilities]
requires = ["events:publish", "filesystem:read"]
optional = ["network:outbound"]

[events]
emits = ["my.event"]
listens = ["app.startup"]
"#;

    #[test]
    fn parses_valid_manifest() {
        let m = PluginManifest::from_str(VALID_TOML).unwrap();
        assert_eq!(m.plugin.name, "my-plugin");
        assert_eq!(m.plugin.version, "1.0.0");
        assert_eq!(m.capabilities.requires.len(), 2);
        assert_eq!(m.events.emits, vec!["my.event"]);
    }

    #[test]
    fn required_capabilities_parse() {
        let m = PluginManifest::from_str(VALID_TOML).unwrap();
        let caps = m.capabilities.required_capabilities().unwrap();
        assert!(caps.contains(&Capability::EventsPublish));
        assert!(caps.contains(&Capability::FilesystemRead));
    }

    #[test]
    fn validate_rejects_empty_name() {
        let bad = r#"
[plugin]
name = ""
version = "1.0.0"
entry_point = "x.dll"
"#;
        let m = PluginManifest::from_str(bad).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn validate_rejects_unknown_capability() {
        let bad = r#"
[plugin]
name = "p"
version = "1.0.0"
entry_point = "p.dll"

[capabilities]
requires = ["not:a:real:capability"]
"#;
        let m = PluginManifest::from_str(bad).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn empty_manifest_sections_default() {
        let minimal = r#"
[plugin]
name = "minimal"
version = "0.1.0"
entry_point = "minimal.dll"
"#;
        let m = PluginManifest::from_str(minimal).unwrap();
        assert!(m.validate().is_ok());
        assert!(m.capabilities.requires.is_empty());
        assert!(m.events.emits.is_empty());
    }
}
