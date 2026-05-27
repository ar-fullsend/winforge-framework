use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    /// List all plugins installed in the current project.
    List,
    /// Show detailed information about a plugin.
    Info {
        /// Plugin name or path to plugin directory.
        name: String,
    },
    /// Validate a plugin.toml manifest.
    Validate {
        /// Path to the plugin directory (must contain plugin.toml).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Create a new plugin skeleton in the current directory.
    New {
        name: String,
    },
}

pub fn run(args: &PluginArgs) -> Result<()> {
    match &args.command {
        PluginCommand::List => list_plugins(),
        PluginCommand::Info { name } => plugin_info(name),
        PluginCommand::Validate { path } => validate_manifest(path),
        PluginCommand::New { name } => new_plugin(name),
    }
}

fn list_plugins() -> Result<()> {
    let plugin_dir = std::path::Path::new("plugins");
    if !plugin_dir.exists() {
        println!("No plugins directory found. Create one with `mkdir plugins`.");
        return Ok(());
    }

    let mut found = 0;
    for entry in std::fs::read_dir(plugin_dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("plugin.toml").exists() {
            let manifest = winforge_core::PluginManifest::load(&path)?;
            println!(
                "  {} v{} — {}",
                manifest.plugin.name,
                manifest.plugin.version,
                manifest.plugin.description.as_deref().unwrap_or("")
            );
            found += 1;
        }
    }

    if found == 0 {
        println!("No plugins installed.");
    }
    Ok(())
}

fn plugin_info(name: &str) -> Result<()> {
    let path = if std::path::Path::new(name).exists() {
        PathBuf::from(name)
    } else {
        PathBuf::from("plugins").join(name)
    };

    let manifest = winforge_core::PluginManifest::load(&path)?;
    println!("Name:        {}", manifest.plugin.name);
    println!("Version:     {}", manifest.plugin.version);
    if let Some(desc) = &manifest.plugin.description {
        println!("Description: {desc}");
    }
    if let Some(authors) = &manifest.plugin.authors {
        println!("Authors:     {}", authors.join(", "));
    }
    println!("Entry point: {}", manifest.plugin.entry_point);

    if !manifest.capabilities.requires.is_empty() {
        println!("Requires:    {}", manifest.capabilities.requires.join(", "));
    }
    if !manifest.events.emits.is_empty() {
        println!("Emits:       {}", manifest.events.emits.join(", "));
    }
    if !manifest.events.listens.is_empty() {
        println!("Listens:     {}", manifest.events.listens.join(", "));
    }
    Ok(())
}

fn validate_manifest(path: &PathBuf) -> Result<()> {
    let manifest = winforge_core::PluginManifest::load(path)?;
    manifest.validate().map_err(anyhow::Error::from)?;
    println!("✓ plugin.toml is valid");
    println!("  Name:    {}", manifest.plugin.name);
    println!("  Version: {}", manifest.plugin.version);
    Ok(())
}

fn new_plugin(name: &str) -> Result<()> {
    use crate::scaffold::{ProjectTemplate, scaffold_project};
    let path = PathBuf::from(name);
    scaffold_project(name, &ProjectTemplate::Plugin, &path)?;
    println!("Plugin skeleton created at ./{name}");
    Ok(())
}
