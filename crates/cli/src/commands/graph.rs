use anyhow::Result;
use colored::*;
use marty_core::workspace_manager::WorkspaceManager;

pub fn execute(manager: &WorkspaceManager) -> Result<()> {
    println!("{}", "Project Dependency Graph:".bold().underline());

    let result = manager
        .get_dependency_graph()
        .map_err(|e| anyhow::anyhow!("Failed to get dependency graph: {}", e))?;

    if result.graph.is_none() {
        println!("No dependency graph available");
        return Ok(());
    }

    let graph = result.graph.as_ref().unwrap();

    if !result.cycles.is_empty() {
        let cycles_description = result
            .cycles
            .iter()
            .map(|cycle| {
                let mut path = cycle.clone();
                if let Some(first) = path.first().cloned() {
                    path.push(first);
                }
                path.join(" -> ")
            })
            .collect::<Vec<_>>()
            .join("; ");

        println!(
            "{} {}",
            "Warning:".yellow().bold(),
            format!("Circular dependencies detected: {}", cycles_description).yellow()
        );
    }

    for (node_index, node_weight) in graph.node_indices().zip(graph.node_weights()) {
        println!("{}", node_weight.blue().bold());

        let mut deps = Vec::new();
        for neighbor in graph.neighbors(node_index) {
            if let Some(dep_name) = graph.node_weight(neighbor) {
                deps.push(dep_name.clone());
            }
        }

        if !deps.is_empty() {
            println!("  {} {}", "depends on:".dimmed(), deps.join(", "));
        } else {
            println!("  {}", "no dependencies".dimmed());
        }
        println!();
    }

    Ok(())
}
