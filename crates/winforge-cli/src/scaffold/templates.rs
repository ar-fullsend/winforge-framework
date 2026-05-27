use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum ProjectTemplate {
    /// Bare-minimum Rust + event bus application.
    Minimal,
    /// Rust core with a WinUI 3 front-end shell (Windows only).
    Desktop,
    /// Background Windows Service.
    Service,
    /// Workflow automation application.
    Workflow,
    /// Plugin library.
    Plugin,
}

/// Scaffold a new WinForge project into `root_dir`.
pub fn scaffold_project(name: &str, template: &ProjectTemplate, root_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(root_dir)
        .with_context(|| format!("creating project directory {}", root_dir.display()))?;

    write_winforge_toml(name, template, root_dir)?;
    write_gitignore(root_dir)?;

    match template {
        ProjectTemplate::Minimal | ProjectTemplate::Service => {
            write_rust_main(name, root_dir)?;
        }
        ProjectTemplate::Desktop => {
            write_rust_main(name, root_dir)?;
            write_ui_shell(name, root_dir)?;
        }
        ProjectTemplate::Workflow => {
            write_rust_main(name, root_dir)?;
            write_example_workflow(name, root_dir)?;
        }
        ProjectTemplate::Plugin => {
            write_plugin_lib(name, root_dir)?;
            write_plugin_manifest(name, root_dir)?;
        }
    }

    Ok(())
}

fn write_winforge_toml(name: &str, template: &ProjectTemplate, dir: &Path) -> Result<()> {
    let template_name = format!("{:?}", template).to_lowercase();
    let contents = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
template = "{template_name}"

[runtime]
log_level = "info"
plugin_dir = "plugins"
workflow_dir = "workflows"

[capabilities]
# Grant capabilities to all plugins this project loads.
# Remove capabilities your project does not need.
grant = [
    "events:publish",
    "events:subscribe",
    "filesystem:read",
]
"#
    );
    std::fs::write(dir.join("winforge.toml"), contents)?;
    Ok(())
}

fn write_gitignore(dir: &Path) -> Result<()> {
    let contents = "/target\n/bin\n/obj\n.env\nwinforge.toml.local\n";
    std::fs::write(dir.join(".gitignore"), contents)?;
    Ok(())
}

fn write_rust_main(name: &str, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    let contents = format!(
        r#"use std::sync::Arc;
use winforge_core::{{ActorSystem, EventBus}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting {name}");

    let event_bus = Arc::new(EventBus::default());
    let actor_system = ActorSystem::new(event_bus.clone());

    // TODO: register plugins, start workflows, spawn actors

    // Wait for Ctrl-C
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");
    actor_system.shutdown().await;
    Ok(())
}}
"#
    );
    std::fs::write(dir.join("src").join("main.rs"), contents)?;

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
winforge-core = {{ git = "https://github.com/ar-fullsend/winforge-framework" }}
tokio = {{ version = "1", features = ["full"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter", "fmt"] }}
anyhow = "1"
"#
    );
    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

fn write_plugin_lib(name: &str, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    let struct_name = pascal_case(name);
    let contents = format!(
        r#"use winforge_plugin::prelude::*;

pub struct {struct_name};

#[async_trait]
impl Plugin for {struct_name} {{
    fn name(&self) -> &str {{ "{name}" }}
    fn version(&self) -> &str {{ "0.1.0" }}

    async fn on_load(&mut self, host: &PluginHost) -> CoreResult<()> {{
        info!("{name} plugin loaded");
        Ok(())
    }}

    async fn on_unload(&mut self) -> CoreResult<()> {{
        info!("{name} plugin unloaded");
        Ok(())
    }}

    fn as_any(&self) -> &dyn std::any::Any {{ self }}
}}
"#
    );
    std::fs::write(dir.join("src").join("lib.rs"), contents)?;

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
winforge-plugin = {{ git = "https://github.com/ar-fullsend/winforge-framework" }}
tracing = "0.1"
async-trait = "0.1"
"#
    );
    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

fn write_plugin_manifest(name: &str, dir: &Path) -> Result<()> {
    let contents = format!(
        r#"[plugin]
name = "{name}"
version = "0.1.0"
description = "A WinForge plugin"
entry_point = "target/release/{name}.dll"

[capabilities]
requires = ["events:publish", "events:subscribe"]
optional = ["filesystem:read"]

[events]
emits = []
listens = []
"#
    );
    std::fs::write(dir.join("plugin.toml"), contents)?;
    Ok(())
}

fn write_example_workflow(name: &str, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir.join("workflows"))?;
    let contents = format!(
        r#"name: {name}-example
version: "1.0"
description: Example workflow for {name}
triggers:
  - type: manual

steps:
  - id: hello
    type: command
    run: echo "Hello from WinForge workflow!"

  - id: notify
    type: notify
    message: "Workflow complete!"
    depends_on: [hello]
"#
    );
    std::fs::write(dir.join("workflows").join("example.workflow.yaml"), contents)?;
    Ok(())
}

fn write_ui_shell(name: &str, dir: &Path) -> Result<()> {
    let ui_dir = dir.join("ui");
    std::fs::create_dir_all(&ui_dir)?;

    let app_proj = format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net9.0-windows10.0.19041.0</TargetFramework>
    <RootNamespace>{}</RootNamespace>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <UseWinUI>true</UseWinUI>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.WindowsAppSDK" Version="1.6.*" />
    <PackageReference Include="CommunityToolkit.Mvvm" Version="8.*" />
  </ItemGroup>
</Project>
"#,
        pascal_case(name)
    );
    std::fs::write(ui_dir.join(format!("{}.csproj", pascal_case(name))), app_proj)?;

    let app_xaml = format!(
        r#"<Application
    x:Class="{}.App"
    xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
    xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
</Application>
"#,
        pascal_case(name)
    );
    std::fs::write(ui_dir.join("App.xaml"), app_xaml)?;

    let main_window = format!(
        r#"<Window
    x:Class="{}.MainWindow"
    xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
    xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
    Title="{name}">
    <StackPanel HorizontalAlignment="Center" VerticalAlignment="Center">
        <TextBlock Text="Welcome to {name}" FontSize="24" />
    </StackPanel>
</Window>
"#,
        pascal_case(name)
    );
    std::fs::write(ui_dir.join("MainWindow.xaml"), main_window)?;

    Ok(())
}

fn pascal_case(s: &str) -> String {
    s.split(['-', '_', ' '])
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
