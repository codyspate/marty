use anyhow::Result;
use colored::*;
use marty_core::workspace_manager::WorkspaceManager;

pub async fn execute(manager: &WorkspaceManager, target: &str) -> Result<()> {
    println!("{} {}", "Execution plan for".bold(), target.cyan());

    // Get execution plan from workspace manager
    let execution_plan = manager
        .get_execution_plan(target)
        .map_err(|e| anyhow::anyhow!("Failed to get execution plan: {}", e))?;

    println!("\n{}:", "Execution order".bold());
    for (i, project) in execution_plan.compatible_projects.iter().enumerate() {
        println!("  {}. {}:{}", i + 1, project, execution_plan.task_name);
    }

    Ok(())
}
