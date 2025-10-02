//! Task execution utilities and color management
//!
//! This module provides high-level task execution functions and consistent
//! project color management for terminal output.

use std::collections::HashMap;

use crate::configs::tasks::TaskConfig;
use crate::execution::runner::TaskRunner;
use crate::types::MartyResult;
use crate::workspace::Workspace;
use colored::*;

/// Get a consistent color for a project name
pub fn get_project_color(project_name: &str) -> Color {
    // Use a simple hash of the project name bytes for consistent colors
    let hash = project_name
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    // Cohesive color palette: vibrant jewel tones that are clearly "label" colors
    // Avoiding conventional log colors (red/yellow/green/blue) while maintaining readability
    let colors = [
        Color::TrueColor {
            r: 147,
            g: 112,
            b: 219,
        }, // Medium slate blue - professional purple
        Color::TrueColor {
            r: 64,
            g: 224,
            b: 208,
        }, // Turquoise - vibrant teal
        Color::TrueColor {
            r: 255,
            g: 140,
            b: 0,
        }, // Dark orange - warm accent
        Color::TrueColor {
            r: 199,
            g: 21,
            b: 133,
        }, // Medium violet red - deep pink
        Color::TrueColor {
            r: 72,
            g: 209,
            b: 204,
        }, // Medium turquoise - aqua
        Color::TrueColor {
            r: 138,
            g: 43,
            b: 226,
        }, // Blue violet - rich purple
    ];

    colors[(hash % colors.len() as u64) as usize]
}

/// Run a task on targets with proper dependency resolution and parallel execution
pub async fn run_task_on_targets(
    task_name: &str,
    targets: &[String],
    workspace: &Workspace,
    all_tasks: &HashMap<String, TaskConfig>,
) -> MartyResult<()> {
    let runner = TaskRunner::new(workspace);
    runner
        .run_task_on_targets(task_name, targets, all_tasks)
        .await
}
