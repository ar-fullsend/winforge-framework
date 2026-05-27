use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod scaffold;

use commands::{
    build::BuildArgs,
    host::HostArgs,
    new::NewArgs,
    plugin::PluginArgs,
    run::RunArgs,
    workflow::WorkflowArgs,
};

#[derive(Debug, Parser)]
#[command(
    name = "winforge",
    version,
    about = "WinForge Framework CLI — build, run, and manage WinForge applications",
    long_about = None,
)]
struct Cli {
    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new WinForge project.
    New(NewArgs),
    /// Run the current project in development mode.
    Run(RunArgs),
    /// Build the project for deployment.
    Build(BuildArgs),
    /// Manage plugins.
    Plugin(PluginArgs),
    /// Manage and run workflows.
    Workflow(WorkflowArgs),
    /// Start the WinForge host — connects to the WinUI 3 shell over named pipes.
    Host(HostArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .compact()
        .init();

    match &cli.command {
        Commands::New(args) => commands::new::run(args),
        Commands::Run(args) => commands::run::run(args),
        Commands::Build(args) => commands::build::run(args),
        Commands::Plugin(args) => commands::plugin::run(args),
        Commands::Workflow(args) => commands::workflow::run(args),
        Commands::Host(args) => commands::host::run(args),
    }
}
