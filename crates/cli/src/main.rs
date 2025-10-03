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
    /// Validate a plugin for publication (for plugin developers)
    Validate {
        /// Path to the plugin binary to validate
        path: PathBuf,
        /// Expected plugin name (should match the name returned by the plugin)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Check if a plugin release exists on GitHub
    CheckRelease {
        /// GitHub repository (e.g., "owner/repo")
        #[arg(short, long)]
        github_repo: String,
        /// Plugin name (for monorepo releases)
        #[arg(short, long)]
        plugin: Option<String>,
        /// Version to check (e.g., "0.2.0")
        #[arg(short, long)]
        version: String,
    },
    /// Generate release naming guide for a plugin
    ReleaseGuide {
        /// Plugin name (e.g., "typescript", "cargo")
        name: String,
        /// Version (e.g., "0.2.0")
        version: String,
        /// GitHub repository (e.g., "owner/repo")
        #[arg(short, long)]
        github_repo: Option<String>,
        /// Use monorepo release format
        #[arg(short, long)]
        monorepo: bool,
    },
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
