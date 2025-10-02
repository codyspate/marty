//! High-level workspace management interface
//! 
//! This module provides the [`WorkspaceManager`] which serves as the primary interface
//! for all workspace operations. It encapsulates workspace initialization, project
//! discovery, task execution, and configuration management.
//! 
//! The WorkspaceManager abstracts away the complexity of:
//! - Loading and merging configuration files
//! - Plugin discovery and WASM runtime management  
//! - Workspace traversal and project detection
//! - Task execution planning and dependency resolution
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use marty_core::workspace_manager::{WorkspaceManager, WorkspaceManagerConfig};
//! use std::path::PathBuf;
//! 
//! # async fn example() -> marty_core::types::MartyResult<()> {
//! let manager = WorkspaceManager::new(WorkspaceManagerConfig {
//!     workspace_root: PathBuf::from("."),
//! }).await?;
//! 
//! // List all projects
//! let projects = manager.list_projects(false)?;
//! 
//! // Get execution plan for a task
//! let plan = manager.get_execution_plan("build")?;
//! 
//! // Run a task
//! manager.run_task("test").await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::configs::{
    project::{parse_project_config, ProjectConfig},
    tasks::{parse_tasks_config, TaskConfig, TasksFileConfig},
    workspace::{parse_workspace_config, WorkspaceConfig},
};
use crate::plugin_runtime::WasmWorkspaceProvider;
use crate::results::{DependencyGraphResult, ProjectInfo, InferredProjectInfo, ProjectListResult};
use crate::task_execution::{resolve_task_execution_plan, TaskExecutionPlan};
use crate::tasks::run_task_on_targets;
use crate::types::{MartyError, MartyResult};
use crate::workspace::{
    build_dependency_graph, traverse_workspace, InferredProject, Workspace, WorkspaceProvider,
};

/// High-level workspace manager that encapsulates all workspace operations
pub struct WorkspaceManager {
    pub workspace: Workspace,
    pub task_configs: TasksFileConfig,
    pub workspace_config: WorkspaceConfig,
}

/// Configuration for initializing a workspace manager
pub struct WorkspaceManagerConfig {
    pub workspace_root: PathBuf,
}



impl WorkspaceManager {
    /// Initialize a new workspace manager from the given workspace root
    pub async fn new(config: WorkspaceManagerConfig) -> MartyResult<Self> {
        // Load workspace configuration
        let workspace_config = Self::load_workspace_config(&config.workspace_root)?;
        
        // Load and merge task configurations
        let task_configs = Self::load_task_configs(&config.workspace_root)?;
        
        // Load workspace providers and initialize workspace
        let workspace = Self::initialize_workspace(config.workspace_root, &workspace_config).await?;
        
        Ok(Self {
            workspace,
            task_configs,
            workspace_config,
        })
    }

    /// List all projects in the workspace
    pub fn list_projects(&self, include_inferred: bool) -> MartyResult<ProjectListResult> {
        let tracked_projects = self
            .workspace
            .projects
            .iter()
            .map(|p| {
                let project_config = self.load_project_config(&p.project_dir);
                ProjectInfo {
                    name: p.name.clone(),
                    path: p.project_dir.clone(),
                    tags: project_config
                        .ok()
                        .and_then(|c| c.tags)
                        .unwrap_or_default(),
                    has_config: p.project_dir.join("marty.yml").exists(),
                }
            })
            .collect();

        let inferred_projects = if include_inferred {
            self.workspace
                .inferred_projects
                .iter()
                .map(|p| InferredProjectInfo {
                    name: p.name.clone(),
                    path: p.project_dir.clone(),
                    discovered_by: p.discovered_by.clone(),
                    is_tracked: self
                        .workspace
                        .projects
                        .iter()
                        .any(|tracked| tracked.name == p.name),
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(ProjectListResult {
            explicit_projects: tracked_projects,
            inferred_projects,
            project_colors: self.get_project_colors(),
        })
    }

    /// Get execution plan for a task
    pub fn get_execution_plan(&self, target: &str) -> MartyResult<TaskExecutionPlan> {
        let (project_filter, task_name) = Self::parse_target(target)?;
        resolve_task_execution_plan(
            &self.workspace,
            &self.task_configs,
            &task_name,
            project_filter.as_deref(),
        )
    }

    /// Execute a task on the workspace
    pub async fn run_task(&self, target: &str) -> MartyResult<()> {
        let execution_plan = self.get_execution_plan(target)?;
        
        if execution_plan.compatible_projects.is_empty() {
            return Err(MartyError::Task(format!(
                "No compatible projects found for task '{}'",
                execution_plan.task_name
            )));
        }

        let task_map = self.build_task_map()?;
        
        run_task_on_targets(
            &execution_plan.task_name,
            &execution_plan.compatible_projects,
            &self.workspace,
            &task_map,
        )
        .await?;

        Ok(())
    }

    /// Get dependency graph information
    pub fn get_dependency_graph(&self) -> MartyResult<DependencyGraphResult> {
        Ok(DependencyGraphResult {
            graph: self.workspace.dep_graph.clone(),
            cycles: self.workspace.dependency_cycles.clone(),
        })
    }

    // Private helper methods
    
    fn load_workspace_config(workspace_root: &Path) -> MartyResult<WorkspaceConfig> {
        let workspace_config_path = workspace_root.join(".marty").join("workspace.yml");
        let content = std::fs::read_to_string(&workspace_config_path).map_err(|e| {
            MartyError::Task(format!(
                "Failed to read workspace config {}: {}",
                workspace_config_path.display(),
                e
            ))
        })?;

        parse_workspace_config(&content).map_err(|e| {
            MartyError::Task(format!(
                "Failed to parse workspace config {}: {}",
                workspace_config_path.display(),
                e
            ))
        })
    }

    fn load_task_configs(workspace_root: &Path) -> MartyResult<TasksFileConfig> {
        let tasks_dir = workspace_root.join(".marty").join("tasks");
        let mut task_configs = Vec::new();

        if tasks_dir.exists() {
            for entry in std::fs::read_dir(&tasks_dir).map_err(|e| {
                MartyError::Task(format!("Failed to read tasks directory {}: {}", tasks_dir.display(), e))
            })? {
                let entry = entry.map_err(|e| MartyError::Task(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                    let content = std::fs::read_to_string(&path).map_err(|e| {
                        MartyError::Task(format!("Failed to read task config {}: {}", path.display(), e))
                    })?;
                    let config: TasksFileConfig = parse_tasks_config(&content).map_err(|e| {
                        MartyError::Task(format!("Failed to parse task config {}: {}", path.display(), e))
                    })?;
                    task_configs.push(config);
                }
            }
        }

        // Merge all task configs
        let mut all_tasks = Vec::new();
        let mut name = None;
        let mut description = None;
        let mut targets = None;
        let mut all_tags = Vec::new();

        for config in &task_configs {
            all_tasks.extend(config.tasks.clone());
            if config.name.is_some() {
                name = config.name.clone();
            }
            if config.description.is_some() {
                description = config.description.clone();
            }
            if config.targets.is_some() {
                targets = config.targets.clone();
            }
            if let Some(tags) = &config.tags {
                all_tags.extend(tags.clone());
            }
        }

        // Remove duplicate tags
        all_tags.sort();
        all_tags.dedup();

        Ok(TasksFileConfig {
            name,
            description,
            tasks: all_tasks,
            targets,
            tags: if all_tags.is_empty() {
                None
            } else {
                Some(all_tags)
            },
        })
    }

    async fn initialize_workspace(
        workspace_root: PathBuf,
        workspace_config: &WorkspaceConfig,
    ) -> MartyResult<Workspace> {
        // Load workspace providers
        let providers = Self::load_workspace_providers(&workspace_root, workspace_config)?;

        // Initialize workspace
        let mut workspace = Workspace {
            root: workspace_root,
            projects: Vec::new(),
            inferred_projects: Vec::new(),
            dep_graph: None,
            dependency_cycles: Vec::new(),
        };

        // Discover projects using plugins
        for plugin in &providers {
            traverse_workspace(plugin.as_ref(), &mut workspace);
        }

        // Build dependency graph
        build_dependency_graph(&mut workspace)
            .map_err(|e| MartyError::Task(format!("Failed to build dependency graph: {}", e)))?;

        Ok(workspace)
    }

    fn load_workspace_providers(
        _workspace_root: &Path,
        workspace_config: &WorkspaceConfig,
    ) -> MartyResult<Vec<Box<dyn WorkspaceProvider>>> {
        let workspace_includes = workspace_config
            .includes
            .as_ref()
            .cloned()
            .unwrap_or_default();
        let workspace_excludes = workspace_config
            .excludes
            .as_ref()
            .cloned()
            .unwrap_or_default();

        let providers = WasmWorkspaceProvider::load_all_from_plugins_dir()
            .map_err(|e| MartyError::Task(format!("Failed to load workspace providers: {}", e)))?
            .into_iter()
            .map(|p| {
                Box::new(ConfigurableWorkspaceProvider::new(
                    Box::new(p) as Box<dyn WorkspaceProvider>,
                    workspace_includes.clone(),
                    workspace_excludes.clone(),
                )) as Box<dyn WorkspaceProvider>
            })
            .collect();
        Ok(providers)
    }

    fn build_task_map(&self) -> MartyResult<HashMap<String, TaskConfig>> {
        let mut task_map = HashMap::new();

        // Add all workspace-level tasks to the map
        for task in &self.task_configs.tasks {
            task_map.insert(task.name.clone(), task.clone());
        }

        // Add project-level tasks with project-specific keys
        for project in &self.workspace.projects {
            let project_config_path = project.project_dir.join("marty.yml");
            if !project_config_path.exists() {
                continue;
            }

            let project_config = self.load_project_config(&project.project_dir)?;

            // Add project-specific tasks with keys like "project_name:task_name"
            if let Some(tasks) = &project_config.tasks {
                for task in tasks {
                    let project_task_key = format!("{}:{}", project.name, task.name);
                    task_map.insert(project_task_key, task.clone());
                }
            }
        }

        Ok(task_map)
    }

    fn load_project_config(&self, project_dir: &Path) -> MartyResult<ProjectConfig> {
        let project_config_path = project_dir.join("marty.yml");
        let content = std::fs::read_to_string(&project_config_path).map_err(|e| {
            MartyError::Task(format!(
                "Failed to read project config {}: {}",
                project_config_path.display(),
                e
            ))
        })?;

        parse_project_config(&content).map_err(|e| {
            MartyError::Task(format!(
                "Failed to parse project config {}: {}",
                project_config_path.display(),
                e
            ))
        })
    }

    fn parse_target(target: &str) -> MartyResult<(Option<String>, String)> {
        if let Some((project, task)) = target.split_once(':') {
            Ok((Some(project.to_string()), task.to_string()))
        } else {
            Ok((None, target.to_string()))
        }
    }

    /// Generate consistent color mapping for projects
    fn get_project_colors(&self) -> HashMap<String, colored::Color> {
        let mut colors = HashMap::new();
        let available_colors = [
            colored::Color::Red,
            colored::Color::Green,
            colored::Color::Yellow,
            colored::Color::Blue,
            colored::Color::Magenta,
            colored::Color::Cyan,
            colored::Color::White,
            colored::Color::BrightRed,
            colored::Color::BrightGreen,
            colored::Color::BrightYellow,
            colored::Color::BrightBlue,
            colored::Color::BrightMagenta,
            colored::Color::BrightCyan,
        ];

        let mut all_projects: Vec<String> = Vec::new();
        all_projects.extend(self.workspace.projects.iter().map(|p| p.name.clone()));
        all_projects.extend(self.workspace.inferred_projects.iter().map(|p| p.name.clone()));
        all_projects.sort();

        for (i, project) in all_projects.iter().enumerate() {
            let color = available_colors[i % available_colors.len()];
            colors.insert(project.clone(), color);
        }

        colors
    }
}

/// Wrapper that combines workspace config includes with plugin includes
struct ConfigurableWorkspaceProvider {
    inner: Box<dyn WorkspaceProvider>,
    workspace_includes: Vec<String>,
    workspace_excludes: Vec<String>,
}

impl ConfigurableWorkspaceProvider {
    fn new(
        inner: Box<dyn WorkspaceProvider>,
        workspace_includes: Vec<String>,
        workspace_excludes: Vec<String>,
    ) -> Self {
        Self {
            inner,
            workspace_includes,
            workspace_excludes,
        }
    }
}

impl WorkspaceProvider for ConfigurableWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // If workspace includes are specified, use only those (don't combine with plugin includes)
        if !self.workspace_includes.is_empty() {
            return self.workspace_includes.clone();
        }

        // Otherwise, use plugin includes
        let plugin_includes = self.inner.include_path_globs();
        if !plugin_includes.is_empty() {
            return plugin_includes;
        }

        // If both are empty, return empty to use defaults
        Vec::new()
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        let mut excludes = self.workspace_excludes.clone();
        let plugin_excludes = self.inner.exclude_path_globs();
        excludes.extend(plugin_excludes);
        excludes
    }

    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        self.inner.on_file_found(workspace, path)
    }
}