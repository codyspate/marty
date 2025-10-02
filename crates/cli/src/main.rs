use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use marty_core::workspace_manager::{WorkspaceManager, WorkspaceManagerConfig};

mod commands;

/// Marty - A monorepo management tool
#[derive(Parser)]
#[command(name = "marty")]
#[command(about = "A powerful monorepo management tool")]
#[command(version)]
struct Cli {
    /// Path to the workspace root (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List projects in the workspace
    List {
        /// Include projects inferred from workspace providers even without a marty.yml
        #[arg(long)]
        inferred: bool,
    },
    /// Show execution plan for a task without running it
    Plan {
        /// Target in format "project:task" or just "task" for all projects
        target: String,
    },
    /// Run a task
    Run {
        /// Target in format "project:task" or just "task" for all projects
        target: String,
    },
    /// Show the project dependency graph
    Graph,
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List cached plugins
    List,
    /// Clear plugin cache
    Clear,
    /// Update all plugins from their URLs
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize workspace manager with all business logic
    let manager = WorkspaceManager::new(WorkspaceManagerConfig {
        workspace_root: cli.workspace,
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize workspace: {}", e))?;

    // Execute command (CLI layer only handles presentation)
    match cli.command {
        Commands::List { inferred } => commands::list::execute(&manager, inferred),
        Commands::Plan { target } => commands::plan::execute(&manager, &target).await,
        Commands::Run { target } => commands::run::execute(&manager, &target).await,
        Commands::Graph => commands::graph::execute(&manager),
        Commands::Plugin { plugin_command } => {
            commands::plugin::execute(&manager, plugin_command).await
        }
    }
}
