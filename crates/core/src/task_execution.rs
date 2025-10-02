use std::fs;

use crate::configs::project::{parse_project_config, ProjectConfig};
use crate::configs::tasks::TasksFileConfig;
use crate::types::{MartyError, MartyResult};
use crate::workspace::{get_recursive_dependencies, Workspace};

/// Result of resolving which projects should execute a task
#[derive(Debug, Clone)]
pub struct TaskExecutionPlan {
    pub task_name: String,
    pub compatible_projects: Vec<String>,
    pub project_filter: Option<String>,
}

/// Check if a project is compatible with a task based on tags
pub fn is_project_compatible_with_task(
    workspace: &Workspace,
    project_name: &str,
    tasks_file_config: &TasksFileConfig,
) -> MartyResult<bool> {
    // If task file has no tags, it's compatible with all projects
    let task_file_tags = tasks_file_config.tags.clone().unwrap_or_default();
    if task_file_tags.is_empty() {
        return Ok(true);
    }

    // Find the project
    let project = workspace.projects.iter().find(|p| p.name == project_name);
    let project = match project {
        Some(p) => p,
        None => return Ok(false), // Project doesn't exist
    };

    // Read project config
    let project_config_path = project.project_dir.join("marty.yml");
    if !project_config_path.exists() {
        // If no project config, assume it's compatible (no tags means no restrictions)
        return Ok(true);
    }

    let content = fs::read_to_string(&project_config_path).map_err(|e| {
        MartyError::Task(format!(
            "Failed to read project config {}: {}",
            project_config_path.display(),
            e
        ))
    })?;

    let project_config: ProjectConfig = parse_project_config(&content).map_err(|e| {
        MartyError::Task(format!(
            "Failed to parse project config {}: {}",
            project_config_path.display(),
            e
        ))
    })?;

    // Get project tags, defaulting to empty if not specified
    let project_tags = project_config.tags.unwrap_or_default();

    // If project has no tags, it's not compatible with tagged tasks
    if project_tags.is_empty() {
        return Ok(false);
    }

    // Check if any task file tag matches any project tag
    let has_matching_tag = task_file_tags.iter().any(|task_tag| {
        project_tags
            .iter()
            .any(|project_tag| project_tag == task_tag)
    });

    Ok(has_matching_tag)
}

/// Check if a task exists either at workspace level or in any project
pub fn task_exists(
    workspace: &Workspace,
    config: &TasksFileConfig,
    task_name: &str,
    project_filter: Option<&str>,
) -> MartyResult<bool> {
    // Check workspace-level tasks
    if config.tasks.iter().any(|t| t.name == task_name) {
        return Ok(true);
    }

    // If we have a specific project filter, only check that project
    if let Some(project_name) = project_filter {
        return check_task_in_project(workspace, project_name, task_name);
    }

    // Check all projects for the task
    for project in &workspace.projects {
        if check_task_in_project(workspace, &project.name, task_name)? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if a specific project has a task
fn check_task_in_project(
    workspace: &Workspace,
    project_name: &str,
    task_name: &str,
) -> MartyResult<bool> {
    let project = workspace
        .projects
        .iter()
        .find(|p| p.name == project_name)
        .ok_or_else(|| MartyError::Task(format!("Project '{}' not found", project_name)))?;

    let project_config_path = project.project_dir.join("marty.yml");
    if !project_config_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&project_config_path).map_err(|e| {
        MartyError::Task(format!(
            "Failed to read project config {}: {}",
            project_config_path.display(),
            e
        ))
    })?;

    let project_config: TasksFileConfig = crate::configs::tasks::parse_tasks_config(&content)
        .map_err(|e| {
            MartyError::Task(format!(
                "Failed to parse project config {}: {}",
                project_config_path.display(),
                e
            ))
        })?;

    Ok(project_config.tasks.iter().any(|t| t.name == task_name))
}

/// Resolve which projects should execute a task, including dependency resolution and tag filtering
pub fn resolve_task_execution_plan(
    workspace: &Workspace,
    config: &TasksFileConfig,
    task_name: &str,
    project_filter: Option<&str>,
) -> MartyResult<TaskExecutionPlan> {
    // Verify the task exists
    if !task_exists(workspace, config, task_name, project_filter)? {
        return Err(MartyError::Task(format!("Task '{}' not found", task_name)));
    }

    // Determine initial targets
    let initial_targets = match project_filter {
        Some(project_name) => {
            // Check if the specific project exists
            if !workspace.projects.iter().any(|p| p.name == project_name) {
                return Err(MartyError::Task(format!("Project '{}' not found", project_name)));
            }
            vec![project_name.to_string()]
        }
        None => {
            // Use all projects as initial targets
            workspace.projects.iter().map(|p| p.name.clone()).collect()
        }
    };

    // Get all projects with their dependencies
    let all_projects_with_deps = get_recursive_dependencies(workspace, &initial_targets)
        .map_err(|e| MartyError::Task(format!("Failed to resolve dependencies: {}", e)))?;

    // Filter projects to only those compatible with the task based on tags
    let mut compatible_projects = Vec::new();
    for project_name in &all_projects_with_deps {
        if is_project_compatible_with_task(workspace, project_name, config)? {
            compatible_projects.push(project_name.clone());
        }
    }

    // If we have a specific project filter, make sure the target project is compatible
    if let Some(target_project) = project_filter {
        if !compatible_projects.contains(&target_project.to_string()) {
            return Err(MartyError::Task(format!(
                "Project '{}' is not compatible with task '{}' (tag mismatch)",
                target_project,
                task_name
            )));
        }
    }

    Ok(TaskExecutionPlan {
        task_name: task_name.to_string(),
        compatible_projects,
        project_filter: project_filter.map(|s| s.to_string()),
    })
}