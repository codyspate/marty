use anyhow::Result;
use colored::*;
use marty_core::workspace_manager::WorkspaceManager;

pub async fn execute(manager: &WorkspaceManager, target: &str) -> Result<()> {
    println!("{} {}", "Running task".bold(), target.cyan());
    println!();

    // Execute task using workspace manager
    manager
        .run_task(target)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run task: {}", e))?;

    println!();
    println!(
        "{} {}",
        "✓".green().bold(),
        "All tasks completed successfully!".green().bold()
    );

    Ok(())
}
