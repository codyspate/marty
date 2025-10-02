use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(about = "pnpm workspace discovery plugin", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    IgnoreGlobs,
    OnFileFound { path: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::IgnoreGlobs => {
            let globs = marty_plugin_pnpm::ignore_path_globs();
            serde_json::to_writer(io::stdout(), &globs)?;
        }
        Commands::OnFileFound { path } => {
            let mut stdin = String::new();
            io::stdin()
                .read_to_string(&mut stdin)
                .context("Failed to read file contents from stdin")?;

            if let Some(project) = marty_plugin_pnpm::process_package_json(&path, &stdin) {
                serde_json::to_writer(io::stdout(), &project)?;
            } else {
                serde_json::to_writer(io::stdout(), &json!(null))?;
            }
        }
    }

    Ok(())
}
