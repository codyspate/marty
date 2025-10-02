//! High-level task runner
//! 
//! This module provides the main task execution logic that coordinates command execution,
//! dependency resolution, and parallel execution.

use std::collections::HashMap;

use colored::*;

use crate::configs::tasks::{Command as TaskCommand, TaskConfig};
use crate::execution::command::CommandExecutor;
use crate::execution::dependencies::group_by_dependency_levels;
use crate::tasks::get_project_color;
use crate::types::{MartyError, MartyResult};
use crate::workspace::{get_recursive_dependencies, Workspace};

/// Configuration for the task runner
#[derive(Debug, Default)]
pub struct TaskRunnerConfig {
    #[allow(dead_code)] // Will be used when parallel execution is implemented
    pub enable_parallel_execution: bool,
}

/// High-level task runner that coordinates task execution across projects
pub struct TaskRunner<'a> {
    workspace: &'a Workspace,
    #[allow(dead_code)] // Will be used when parallel execution is implemented
    config: TaskRunnerConfig,
}

impl<'a> TaskRunner<'a> {
    pub fn new(workspace: &'a Workspace) -> Self {
        Self {
            workspace,
            config: TaskRunnerConfig::default(),
        }
    }



    /// Run a task on targets with proper dependency resolution and parallel execution
    pub async fn run_task_on_targets(
        &self,
        task_name: &str,
        targets: &[String],
        all_tasks: &HashMap<String, TaskConfig>,
    ) -> MartyResult<()> {
        // Verify the base task exists (either workspace-level or at least one project has it)
        let base_task_exists = all_tasks.contains_key(task_name)
            || all_tasks
                .keys()
                .any(|key| key.ends_with(&format!(":{}", task_name)));

        if !base_task_exists {
            return Err(MartyError::Task(format!("Task '{}' not found", task_name)));
        }

        // Get all projects that need this task run on them (targets + their dependencies)
        let all_projects = get_recursive_dependencies(self.workspace, targets).map_err(MartyError::Task)?;

        // Group projects by dependency level (topological levels)
        let levels = group_by_dependency_levels(self.workspace, &all_projects)?;

        // Execute tasks level by level
        for level in levels {
            // Within each level, run tasks sequentially for now
            // TODO: Implement parallel execution within levels when config.enable_parallel_execution is true
            for project_name in level {
                self.run_task_on_project(task_name, &project_name, all_tasks).await?;
            }
        }

        Ok(())
    }

    /// Run a task on a single project
    async fn run_task_on_project(
        &self,
        task_name: &str,
        project_name: &str,
        all_tasks: &HashMap<String, TaskConfig>,
    ) -> MartyResult<()> {
        // Resolve task config for this specific project (project-level overrides workspace-level)
        let project_task_key = format!("{}:{}", project_name, task_name);
        let (task_config, is_project_override) =
            if let Some(project_task) = all_tasks.get(&project_task_key) {
                // Use project-specific task if it exists
                (project_task, true)
            } else if let Some(workspace_task) = all_tasks.get(task_name) {
                // Fall back to workspace-level task
                (workspace_task, false)
            } else {
                return Err(MartyError::Task(format!(
                    "Task '{}' not found for project '{}'",
                    task_name, project_name
                )));
            };

        // Print task execution header with colors
        let project_color = get_project_color(project_name);
        let task_source = if is_project_override {
            "project".bright_blue()
        } else {
            "workspace".bright_black()
        };

        println!();
        println!(
            "┌─ {} {}",
            format!("Running task '{}'", task_name).bold(),
            format!("on {}", project_name).color(project_color).bold()
        );
        println!("└─ {} {}", "Source:".bright_black(), task_source);

        self.run_task(task_config, &[project_name.to_string()], all_tasks)
    }

    /// Execute a single task with dependency handling
    fn run_task(
        &self,
        task_config: &TaskConfig,
        targets: &[String],
        all_tasks: &HashMap<String, TaskConfig>,
    ) -> MartyResult<()> {
        // Handle dependencies first
        if let Some(deps) = &task_config.dependencies {
            for dep_name in deps {
                if let Some(dep_task) = all_tasks.get(dep_name) {
                    self.run_task(dep_task, targets, all_tasks)?;
                } else {
                    return Err(MartyError::Task(format!(
                        "Dependency '{}' not found for task '{}'",
                        dep_name, task_config.name
                    )));
                }
            }
        }

        // Determine the effective targets for this task
        let effective_targets = task_config.override_targets.as_deref().unwrap_or(targets);

        // Execute the task based on its configuration
        let executor = CommandExecutor::new(self.workspace, effective_targets);
        
        if let Some(script) = &task_config.script {
            executor.execute_script(script)?;
        } else if let Some(command) = &task_config.command {
            self.execute_task_command(&executor, command)?;
        } else {
            return Err(MartyError::Task(format!(
                "Task '{}' has no script or command to execute",
                task_config.name
            )));
        }

        Ok(())
    }

    /// Execute a task command (single or multiple)
    fn execute_task_command(&self, executor: &CommandExecutor, command: &TaskCommand) -> MartyResult<()> {
        match command {
            TaskCommand::Single(cmd) => executor.execute_shell_command(cmd),
            TaskCommand::Multiple(cmds) => {
                if cmds.is_empty() {
                    return Ok(());
                }
                // Execute the first command with the rest as arguments
                executor.execute_command_with_args(&cmds[0], &cmds[1..])
            }
        }
    }
}