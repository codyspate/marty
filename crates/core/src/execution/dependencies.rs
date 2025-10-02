//! Task dependency management
//!
//! This module handles the resolution of task dependencies, including topological sorting
//! and dependency level grouping for parallel execution.

use std::collections::HashMap;

use crate::types::{MartyError, MartyResult};
use crate::workspace::Workspace;

/// Group projects by their dependency levels (topological levels)
pub fn group_by_dependency_levels(
    workspace: &Workspace,
    projects: &[String],
) -> MartyResult<Vec<Vec<String>>> {
    if workspace.dep_graph.is_none() {
        return Err(MartyError::Task("Dependency graph not built".to_string()));
    }

    let graph = workspace.dep_graph.as_ref().unwrap();

    // Create a mapping from project names to node indices
    let mut name_to_node = HashMap::new();
    for (node_index, node_weight) in graph.node_indices().zip(graph.node_weights()) {
        name_to_node.insert(node_weight.clone(), node_index);
    }

    // Calculate levels using topological sort
    let mut levels = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut current_level = Vec::new();

    // Start with projects that have no dependencies (leaf nodes in reverse topological order)
    for project in projects {
        if let Some(&node_index) = name_to_node.get(project) {
            if graph.neighbors(node_index).count() == 0 {
                current_level.push(project.clone());
                visited.insert(node_index);
            }
        }
    }

    while !current_level.is_empty() {
        levels.push(current_level.clone());
        let mut next_level = Vec::new();

        // Find projects that depend on the current level
        for project in &current_level {
            if let Some(&node_index) = name_to_node.get(project) {
                // Find all projects that have this as a dependency (reverse edges)
                for potential_parent in graph.node_indices() {
                    if graph.contains_edge(potential_parent, node_index) {
                        let parent_name = graph.node_weight(potential_parent).unwrap();
                        if projects.contains(parent_name) && !visited.contains(&potential_parent) {
                            next_level.push(parent_name.clone());
                            visited.insert(potential_parent);
                        }
                    }
                }
            }
        }

        current_level = next_level
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
    }

    // Reverse to get dependencies first
    levels.reverse();
    Ok(levels)
}
