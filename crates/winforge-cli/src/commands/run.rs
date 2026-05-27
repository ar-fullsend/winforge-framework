use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Enable hot-reload (watches source files and restarts on changes).
    #[arg(long)]
    pub hot_reload: bool,

    /// Log level override (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Path to the project manifest (default: ./winforge.toml).
    #[arg(long, default_value = "winforge.toml")]
    pub manifest: String,
}

pub fn run(args: &RunArgs) -> Result<()> {
    println!("Starting WinForge dev server...");
    println!("  Manifest:   {}", args.manifest);
    println!("  Log level:  {}", args.log_level);
    println!("  Hot reload: {}", args.hot_reload);
    println!();

    // Check the manifest exists.
    if !std::path::Path::new(&args.manifest).exists() {
        anyhow::bail!(
            "no winforge.toml found in the current directory. \
             Run `winforge new <name>` to create a new project."
        );
    }

    // In a full implementation this would:
    // 1. Parse winforge.toml
    // 2. Load plugins from the plugin_dir
    // 3. Watch workflow_dir for .workflow.yaml files
    // 4. Start the Rust process via `cargo run`
    // 5. If --hot-reload, watch src/ for changes and restart

    println!("Running: cargo run");
    let status = std::process::Command::new("cargo")
        .arg("run")
        .env("RUST_LOG", &args.log_level)
        .status()?;

    if !status.success() {
        anyhow::bail!("cargo run exited with {status}");
    }
    Ok(())
}
