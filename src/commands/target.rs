use crate::target::{AddResult, RemoveResult, TargetKind, TargetRegistry};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List configured targets
    #[command(
        long_about = "Display all registered target environments. Shows which AI coding assistants (codex, copilot) are configured to receive plugin deployments."
    )]
    List,

    /// Add a target environment
    #[command(
        long_about = "Register a new target environment. Available targets: codex, copilot. Once added, plugins can be deployed to this target."
    )]
    Add {
        #[arg(value_enum)]
        target: TargetKind,
    },

    /// Remove a target environment
    #[command(
        long_about = "Unregister a target environment. Plugins will no longer be deployed to this target, but existing deployments are not removed."
    )]
    Remove {
        #[arg(value_enum)]
        target: TargetKind,
    },
}

pub async fn run(args: Args) -> Result<(), String> {
    let mut registry = TargetRegistry::new().map_err(|e| e.to_string())?;

    match args.command {
        Command::List => {
            let targets = registry.list().map_err(|e| e.to_string())?;

            if targets.is_empty() {
                println!("No targets registered. Use 'plm target add <target>' to add one.");
            } else {
                println!("Registered Targets:");
                for target in targets {
                    println!("  - {}", target.as_str());
                }
            }
            Ok(())
        }
        Command::Add { target } => {
            match registry.add(target).map_err(|e| e.to_string())? {
                AddResult::Added => println!("Target added: {}", target.as_str()),
                AddResult::AlreadyExists => {
                    println!("Target already exists: {}", target.as_str())
                }
            }
            Ok(())
        }
        Command::Remove { target } => {
            match registry.remove(target).map_err(|e| e.to_string())? {
                RemoveResult::Removed => println!("Target removed: {}", target.as_str()),
                RemoveResult::NotFound => println!("Target not found: {}", target.as_str()),
            }
            Ok(())
        }
    }
}
