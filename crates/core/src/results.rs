//! Result types for workspace operations
//! 
//! This module contains all result types returned by workspace manager operations,
//! providing a centralized location for output structures.

use std::collections::HashMap;
use std::path::PathBuf;

use colored::Color;

use crate::task_execution::TaskExecutionPlan;
use crate::workspace::{InferredProject, Project};

/// Information about a tracked project with its configuration
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub has_config: bool,
}

/// Information about an inferred project discovered by plugins
#[derive(Debug, Clone)]
pub struct InferredProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub discovered_by: String,
    pub is_tracked: bool,
}

/// Result of listing projects in the workspace
#[derive(Debug)]
pub struct ProjectListResult {
    pub explicit_projects: Vec<ProjectInfo>,
    pub inferred_projects: Vec<InferredProjectInfo>,
    pub project_colors: HashMap<String, Color>,
}

/// Result of getting the dependency graph
#[derive(Debug)]
pub struct DependencyGraphResult {
    pub graph: Option<petgraph::Graph<String, ()>>,
    pub cycles: Vec<Vec<String>>,
}

/// Result of task execution planning
#[derive(Debug)]
pub struct TaskPlanResult {
    pub plan: TaskExecutionPlan,
    pub project_colors: HashMap<String, Color>,
}

impl From<Project> for ProjectInfo {
    fn from(project: Project) -> Self {
        Self {
            name: project.name,
            path: project.project_dir,
            tags: Vec::new(), // Will be populated by caller with config data
            has_config: false, // Will be populated by caller
        }
    }
}

impl From<InferredProject> for InferredProjectInfo {
    fn from(project: InferredProject) -> Self {
        Self {
            name: project.name,
            path: project.project_dir,
            discovered_by: project.discovered_by,
            is_tracked: false, // Will be populated by caller
        }
    }
}