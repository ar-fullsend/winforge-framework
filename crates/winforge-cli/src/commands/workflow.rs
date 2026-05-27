use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub command: WorkflowCommand,
}

#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// List all workflow definitions in the current project.
    List,
    /// Validate a workflow YAML file.
    Validate {
        /// Path to the .workflow.yaml file.
        path: PathBuf,
    },
    /// Run a workflow immediately (dev/testing mode).
    Run {
        /// Path to the .workflow.yaml file.
        path: PathBuf,
        /// JSON string of context variables (e.g. '{"key":"value"}').
        #[arg(long, default_value = "{}")]
        context: String,
    },
}

pub fn run(args: &WorkflowArgs) -> Result<()> {
    match &args.command {
        WorkflowCommand::List => list_workflows(),
        WorkflowCommand::Validate { path } => validate_workflow(path),
        WorkflowCommand::Run { path, context } => run_workflow(path, context),
    }
}

fn list_workflows() -> Result<()> {
    let workflow_dir = std::path::Path::new("workflows");
    if !workflow_dir.exists() {
        println!("No workflows directory found.");
        return Ok(());
    }

    let mut found = 0;
    for entry in glob_yaml(workflow_dir)? {
        let def = winforge_workflow::WorkflowDefinition::load(&entry)?;
        println!(
            "  {} v{} ({} steps) — {}",
            def.name,
            def.version,
            def.steps.len(),
            def.description.as_deref().unwrap_or(""),
        );
        found += 1;
    }

    if found == 0 {
        println!("No workflow files found (*.workflow.yaml).");
    }
    Ok(())
}

fn validate_workflow(path: &PathBuf) -> Result<()> {
    let def = winforge_workflow::WorkflowDefinition::load(path)?;
    println!("✓ Workflow '{}' is valid ({} steps)", def.name, def.steps.len());
    Ok(())
}

fn run_workflow(path: &PathBuf, context_json: &str) -> Result<()> {
    let def = winforge_workflow::WorkflowDefinition::load(path)?;
    let context: HashMap<String, serde_json::Value> = serde_json::from_str(context_json)?;

    println!("Running workflow '{}' ({} steps)...", def.name, def.steps.len());

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let bus = Arc::new(winforge_core::EventBus::default());
        let engine = winforge_workflow::WorkflowEngine::new(bus);
        match engine.run(&def, context).await {
            Ok(instance) => {
                println!("Workflow completed: {:?}", instance.status);
                for (step_id, result) in &instance.steps {
                    println!("  [{:?}] {step_id}", result.status);
                }
            }
            Err(e) => eprintln!("Workflow failed: {e}"),
        }
    });

    Ok(())
}

fn glob_yaml(dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut paths = vec![];
    for entry in std::fs::read_dir(dir)?.flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            paths.push(p);
        }
    }
    Ok(paths)
}
