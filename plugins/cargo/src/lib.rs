use std::collections::HashSet;
use std::path::Path;

use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, InferredProjectMessage, MartyPlugin, PluginType,
    Workspace, WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};
use toml::Value;

/// Main Cargo plugin struct
pub struct CargoPlugin;

/// Workspace provider for Cargo projects  
pub struct CargoWorkspaceProvider;

impl Default for CargoPlugin {
    fn default() -> Self {
        Self
    }
}

impl CargoPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl WorkspaceProvider for CargoWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/Cargo.toml".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec!["**/target/**".to_string()]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        if path.file_name()?.to_str()? != "Cargo.toml" {
            return None;
        }

        let contents = std::fs::read_to_string(path).ok()?;
        let message = process_manifest(path, &contents)?;

        Some(InferredProject {
            name: message.name,
            project_dir: std::path::PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        })
    }
}

impl MartyPlugin for CargoPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary
    }

    fn name(&self) -> &str {
        "Cargo Plugin"
    }

    fn key(&self) -> &str {
        "cargo"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &CargoWorkspaceProvider
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
export_plugin!(CargoPlugin);

pub fn process_manifest(
    manifest_path: &Path,
    manifest_contents: &str,
) -> Option<InferredProjectMessage> {
    if manifest_path.file_name()?.to_str()? != "Cargo.toml" {
        return None;
    }

    let project_dir = manifest_path.parent()?.to_path_buf();
    let manifest = parse_manifest(manifest_contents, &project_dir)?;

    Some(InferredProjectMessage::new(
        manifest.package_name,
        project_dir.display().to_string(),
        "cargo",
        manifest.workspace_dependencies,
    ))
}

struct ParsedManifest {
    package_name: String,
    workspace_dependencies: Vec<String>,
}

fn parse_manifest(manifest_contents: &str, project_dir: &Path) -> Option<ParsedManifest> {
    let manifest_value: Value = toml::from_str(manifest_contents).ok()?;

    let package_name = manifest_value
        .get("package")
        .and_then(Value::as_table)
        .and_then(|pkg| pkg.get("name"))
        .and_then(Value::as_str)
        .map(|name| name.to_string())
        .or_else(|| {
            project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })?;

    let workspace_dependencies = collect_workspace_dependencies(&manifest_value, project_dir);

    Some(ParsedManifest {
        package_name,
        workspace_dependencies,
    })
}

fn collect_workspace_dependencies(manifest: &Value, project_dir: &Path) -> Vec<String> {
    let mut dependencies = HashSet::new();

    if let Some(table) = manifest.get("dependencies").and_then(Value::as_table) {
        for (dep_name, dep_value) in table {
            if let Some(resolved_name) = resolve_dependency_name(dep_name, dep_value, project_dir) {
                dependencies.insert(resolved_name);
            }
        }
    }

    let mut dependency_list: Vec<String> = dependencies.into_iter().collect();
    dependency_list.sort_unstable();
    dependency_list
}

fn resolve_dependency_name(
    dep_key: &str,
    dep_value: &Value,
    _project_dir: &Path,
) -> Option<String> {
    match dep_value {
        Value::Table(details) => {
            // Only consider dependencies with a "path" attribute as workspace dependencies
            if details.get("path").is_some() {
                // For path dependencies, use the dependency key (package name) directly
                // This is the actual package name that should be used for dependencies
                Some(dep_key.to_string())
            } else {
                // Dependencies with only "workspace = true" are external crates with inherited versions,
                // not workspace members
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extracts_path_dependencies_and_package_name() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let root = temp_dir.path();

        let app_dir = root.join("app");
        let lib_dir = root.join("lib");

        assert!(std::fs::create_dir_all(&app_dir).is_ok());
        assert!(std::fs::create_dir_all(&lib_dir).is_ok());

        let lib_manifest = r#"
[package]
name = "lib-crate"
version = "0.1.0"
"#;
        let app_manifest = r#"
[package]
name = "app-crate"
version = "0.1.0"

[dependencies]
lib-crate = { path = "../lib" }
"#;

        assert!(std::fs::write(lib_dir.join("Cargo.toml"), lib_manifest).is_ok());
        assert!(std::fs::write(app_dir.join("Cargo.toml"), app_manifest).is_ok());

        let manifest_contents = std::fs::read_to_string(app_dir.join("Cargo.toml")).unwrap();
        let inferred = process_manifest(&app_dir.join("Cargo.toml"), &manifest_contents)
            .expect("project should be inferred");

        assert_eq!(inferred.name, "app-crate");
        assert_eq!(inferred.project_dir, app_dir.display().to_string());
        assert_eq!(
            inferred.workspace_dependencies,
            vec!["lib-crate".to_string()]
        );
    }
}
