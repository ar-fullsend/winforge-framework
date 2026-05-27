use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use tracing::info;

#[derive(Debug, Args)]
pub struct HostArgs {
    /// Directory to scan for workflow YAML files.
    #[arg(long, default_value = "workflows")]
    pub workflows_dir: String,

    /// Directory to scan for plugins.
    #[arg(long, default_value = "plugins")]
    pub plugins_dir: String,
}

pub fn run(args: &HostArgs) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(args))
}

async fn run_async(args: &HostArgs) -> Result<()> {
    let runtime = Arc::new(winforge_host::HostRuntime::new());

    info!("WinForge host starting");
    info!("  Workflows dir : {}", args.workflows_dir);
    info!("  Plugins dir   : {}", args.plugins_dir);
    info!("  cmd pipe      : \\\\.\\pipe\\winforge-shell-cmd");
    info!("  evt pipe      : \\\\.\\pipe\\winforge-shell-evt");

    // Discover plugins if the directory exists.
    if std::path::Path::new(&args.plugins_dir).exists() {
        let mut plugins = runtime.plugins.lock().await;
        let n = plugins.discover(std::path::Path::new(&args.plugins_dir)).await?;
        info!("loaded {n} plugins from '{}'", args.plugins_dir);
    }

    println!();
    println!("WinForge host is running.");
    println!("Open WinForgeShell.exe to connect the UI.");
    println!("Press Ctrl-C to stop.");
    println!();

    let rt2 = runtime.clone();
    tokio::select! {
        result = winforge_host::run_bridge(rt2) => {
            if let Err(e) = result {
                eprintln!("Bridge error: {e}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("received Ctrl-C, shutting down");
        }
    }

    Arc::try_unwrap(runtime)
        .unwrap_or_else(|arc| {
            // Other Arc references still exist — create a fresh shutdown.
            // This shouldn't happen in normal flow.
            std::mem::forget(arc);
            winforge_host::HostRuntime::new()
        })
        .shutdown()
        .await;

    Ok(())
}
