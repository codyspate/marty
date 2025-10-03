use std::collections::HashSet;
use std::path::Path;

use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, InferredProjectMessage, MartyPlugin, PluginType,
    Workspace, WorkspaceProvider,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    dependencies: serde_json::Map<String, serde_json::Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: serde_json::Map<String, serde_json::Value>,
    #[serde(default, rename = "optionalDependencies")]
    optional_dependencies: serde_json::Map<String, serde_json::Value>,
    #[serde(default, rename = "peerDependencies")]
    peer_dependencies: serde_json::Map<String, serde_json::Value>,
}

/// Main PNPM plugin struct
pub struct PnpmPlugin;

/// Workspace provider for PNPM projects  
pub struct PnpmWorkspaceProvider;

impl Default for PnpmPlugin {
    fn default() -> Self {
        Self
    }
}

impl PnpmPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl WorkspaceProvider for PnpmWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/package.json".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/target/**".to_string(),
        ]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        if path.file_name()?.to_str()? != "package.json" {
            return None;
        }

        let contents = std::fs::read_to_string(path).ok()?;
        let message = process_package_json(path, &contents)?;

        Some(InferredProject {
            name: message.name,
            project_dir: std::path::PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        })
    }
}

impl MartyPlugin for PnpmPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary
    }

    fn name(&self) -> &str {
        "PNPM Plugin"
    }

    fn key(&self) -> &str {
        "pnpm"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &PnpmWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<JsonValue> {
        Some(json!({
            "type": "object",
            "properties": {
                "includes": {
                    "type": "array",
                    "description": "Additional glob patterns to include in scanning",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "excludes": {
                    "type": "array",
                    "description": "Additional glob patterns to exclude from scanning",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin using the dynamic library interface
export_plugin!(PnpmPlugin);

pub fn ignore_path_globs() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/target/**".to_string(),
    ]
}

pub fn process_package_json(
    manifest_path: &Path,
    manifest_contents: &str,
) -> Option<InferredProjectMessage> {
    if manifest_path.file_name()?.to_str()? != "package.json" {
        return None;
    }

    let manifest: PackageJson = serde_json::from_str(manifest_contents).ok()?;
    let project_dir = manifest_path.parent()?.to_path_buf();

    let name = manifest.name.clone().or_else(|| {
        project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    })?;

    let dependencies = gather_workspace_dependencies(&manifest);

    Some(InferredProjectMessage::new(
        name,
        project_dir.display().to_string(),
        "pnpm",
        dependencies,
    ))
}

fn gather_workspace_dependencies(manifest: &PackageJson) -> Vec<String> {
    let mut names = HashSet::new();
    for map in [
        &manifest.dependencies,
        &manifest.dev_dependencies,
        &manifest.optional_dependencies,
        &manifest.peer_dependencies,
    ] {
        for (dep_name, dep_value) in map {
            if let Some(value_str) = dep_value.as_str() {
                if value_str.starts_with("workspace:") || value_str.starts_with("file:") {
                    names.insert(dep_name.clone());
                }
            }
        }
    }

    let mut result: Vec<String> = names.into_iter().collect();
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extracts_workspace_dependencies() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("web-app");
        std::fs::create_dir_all(&project_dir).unwrap();

        let manifest = r#"
{
  "name": "web-app",
  "dependencies": {
    "shared": "workspace:^",
    "react": "18.2.0"
  },
  "devDependencies": {
    "builder": "file:../builder"
  }
}
"#;

        let message = process_package_json(&project_dir.join("package.json"), manifest)
            .expect("should produce inferred project");

        assert_eq!(message.name, "web-app");
        assert_eq!(message.discovered_by, "pnpm");
        assert_eq!(message.project_dir, project_dir.display().to_string());
        assert_eq!(message.workspace_dependencies, vec!["builder", "shared"]);
    }
}
