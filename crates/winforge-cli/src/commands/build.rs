use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Build configuration.
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Target triple (e.g. x86_64-pc-windows-msvc).
    #[arg(long)]
    pub target: Option<String>,

    /// Package into an MSIX installer after building.
    #[arg(long)]
    pub msix: bool,
}

pub fn run(args: &BuildArgs) -> Result<()> {
    println!("Building WinForge project...");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    if args.profile == "release" {
        cmd.arg("--release");
    } else {
        cmd.args(["--profile", &args.profile]);
    }

    if let Some(target) = &args.target {
        cmd.args(["--target", target]);
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("cargo build failed with {status}");
    }

    if args.msix {
        println!("MSIX packaging: not yet implemented in v0.1");
    }

    println!("Build complete.");
    Ok(())
}
