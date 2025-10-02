use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(about = "Cargo workspace discovery plugin", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print include path globs as JSON array
    IncludeGlobs,
    /// Evaluate a file discovered during workspace traversal
    OnFileFound {
        /// Absolute path to the candidate file
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::IncludeGlobs => {
            let globs = marty_plugin_cargo::include_path_globs();
            serde_json::to_writer(io::stdout(), &globs)?;
        }
        Commands::OnFileFound { path } => {
            let mut stdin = String::new();
            io::stdin()
                .read_to_string(&mut stdin)
                .context("Failed to read file contents from stdin")?;

            if let Some(project) = marty_plugin_cargo::process_manifest(&path, &stdin) {
                serde_json::to_writer(io::stdout(), &project)?;
            } else {
                // Emit explicit null to signal no discovery
                serde_json::to_writer(io::stdout(), &json!(null))?;
            }
        }
    }

    Ok(())
}
