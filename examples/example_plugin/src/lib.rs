//! Example plugin demonstrating the MartyPlugin trait
//!
//! This plugin discovers Python projects by looking for requirements.txt files.
//! It demonstrates how workspace_dependencies should represent dependencies between
//! projects in the same workspace (for task ordering), not external packages.

use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, MartyPlugin, Workspace, WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};
use std::path::Path;

/// Our main plugin struct
pub struct PythonPlugin;

impl PythonPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PythonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// The workspace provider implementation for Python projects
pub struct PythonWorkspaceProvider;

impl WorkspaceProvider for PythonWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec![
            "**/requirements.txt".to_string(),
            "**/pyproject.toml".to_string(),
            "**/setup.py".to_string(),
        ]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![
            "**/__pycache__/**".to_string(),
            "**/.venv/**".to_string(),
            "**/venv/**".to_string(),
            "**/*.egg-info/**".to_string(),
        ]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        // Only process requirements.txt files for this example
        if path.file_name()?.to_str()? != "requirements.txt" {
            return None;
        }

        let project_dir = path.parent()?.to_path_buf();
        let name = project_dir.file_name()?.to_str()?.to_string();

        // For workspace dependencies, we would need to look for references to other projects
        // in the same workspace. For example, if this was a setup.py or pyproject.toml,
        // we might find local path dependencies like:
        // - "my-other-project @ file://../my-other-project"
        // - Local editable installs in requirements.txt: "-e ../shared-lib"
        //
        // For this simple example with requirements.txt, we'll assume no workspace dependencies
        let workspace_dependencies = Vec::new();

        // In a real implementation, you might:
        // 1. Parse requirements.txt for "-e ../other-project" entries
        // 2. Check setup.py/pyproject.toml for local path dependencies
        // 3. Look for references to other projects in the same workspace

        Some(InferredProject {
            name,
            project_dir,
            discovered_by: "python-plugin".to_string(),
            workspace_dependencies,
        })
    }
}

impl MartyPlugin for PythonPlugin {
    fn name(&self) -> &str {
        "Python Requirements Plugin"
    }

    fn key(&self) -> &str {
        "python"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &PythonWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<JsonValue> {
        Some(json!({
            "properties": {
                "python_version": {
                    "type": "string",
                    "description": "Required Python version",
                    "default": "3.8"
                },
                "virtual_env_dir": {
                    "type": "string",
                    "description": "Virtual environment directory name",
                    "default": "venv"
                },
                "include_dev_dependencies": {
                    "type": "boolean",
                    "description": "Include development dependencies",
                    "default": false
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin using the dynamic library interface
export_plugin!(PythonPlugin);
