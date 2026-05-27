use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::scaffold::{ProjectTemplate, scaffold_project};

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Name of the new project.
    pub name: String,

    /// Project template to scaffold.
    #[arg(long, short, value_enum, default_value = "minimal")]
    pub template: ProjectTemplate,

    /// Directory to create the project in (defaults to `./<name>`).
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

pub fn run(args: &NewArgs) -> Result<()> {
    let target_dir = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(&args.name));

    if target_dir.exists() {
        anyhow::bail!(
            "directory '{}' already exists; choose a different name or pass --output",
            target_dir.display()
        );
    }

    println!("Creating WinForge project '{}'...", args.name);
    scaffold_project(&args.name, &args.template, &target_dir)?;

    println!();
    println!("  Project created at: {}", target_dir.display());
    println!("  Template:           {:?}", args.template);
    println!();
    println!("Next steps:");
    println!("  cd {}", args.name);
    println!("  winforge run");
    Ok(())
}
