use std::fs;
use std::path::{Path, PathBuf};

use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, InferredProjectMessage, MartyPlugin, Workspace,
    WorkspaceProvider,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Deserialize, Serialize, Default)]
struct TsConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    references: Vec<TsReference>,
    #[serde(flatten)]
    other: serde_json::Map<String, JsonValue>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct TsReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

/// Main TypeScript plugin struct
pub struct TypeScriptPlugin;

/// Workspace provider for TypeScript projects  
pub struct TypeScriptWorkspaceProvider;

impl Default for TypeScriptPlugin {
    fn default() -> Self {
        Self
    }
}

impl TypeScriptPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl WorkspaceProvider for TypeScriptWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/tsconfig.json".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/dist/**".to_string(),
        ]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        if path.file_name()?.to_str()? != "tsconfig.json" {
            return None;
        }

        let contents = std::fs::read_to_string(path).ok()?;
        let message = process_tsconfig(path, &contents)?;

        Some(InferredProject {
            name: message.name,
            project_dir: std::path::PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        })
    }
}

impl MartyPlugin for TypeScriptPlugin {
    fn name(&self) -> &str {
        "TypeScript Plugin"
    }

    fn key(&self) -> &str {
        "typescript"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &TypeScriptWorkspaceProvider
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
                },
                "auto_project_references": {
                    "type": "boolean",
                    "description": "Automatically add project references to tsconfig.json files based on workspace dependencies detected by other plugins (e.g., PNPM)",
                    "default": false
                },
                "reference_path_style": {
                    "type": "string",
                    "description": "Style for generating project reference paths",
                    "enum": ["relative", "tsconfig"],
                    "default": "relative"
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin using the dynamic library interface
export_plugin!(TypeScriptPlugin);

pub fn ignore_path_globs() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/dist/**".to_string(),
    ]
}

pub fn process_tsconfig(manifest_path: &Path, contents: &str) -> Option<InferredProjectMessage> {
    if manifest_path.file_name()?.to_str()? != "tsconfig.json" {
        return None;
    }

    // Parse tsconfig.json to validate it's a valid TypeScript project
    let _config: TsConfig = serde_json::from_str(contents).ok()?;
    let project_dir = manifest_path.parent()?.to_path_buf();

    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())?;

    // TypeScript plugin only detects projects, workspace dependencies should come from PNPM plugin
    Some(InferredProjectMessage::new(
        name,
        project_dir.display().to_string(),
        "typescript",
        vec![], // No workspace dependencies - let PNPM plugin handle this
    ))
}

/// Update tsconfig.json with project references based on workspace dependencies detected by other plugins
/// This function takes workspace dependencies (typically from PNPM plugin) and converts them to TypeScript project references
pub fn update_project_references(
    tsconfig_path: &Path,
    workspace_dependencies: &[String],
    workspace: &Workspace,
    reference_path_style: &str,
) -> anyhow::Result<bool> {
    // Read and parse existing tsconfig.json
    let contents = fs::read_to_string(tsconfig_path)?;
    let mut config: TsConfig = serde_json::from_str(&contents).unwrap_or_default();

    // Calculate project references based on workspace dependencies
    let mut new_references = Vec::new();

    for dep_name in workspace_dependencies {
        // Find the dependency project in the workspace
        let dep_project = workspace
            .projects
            .iter()
            .find(|p| &p.name == dep_name)
            .map(|p| &p.project_dir)
            .or_else(|| {
                workspace
                    .inferred_projects
                    .iter()
                    .find(|p| &p.name == dep_name)
                    .map(|p| &p.project_dir)
            });

        if let Some(dep_dir) = dep_project {
            let current_dir = tsconfig_path.parent().unwrap_or_else(|| Path::new("."));

            // Calculate relative path to the dependency
            let relative_path = match pathdiff::diff_paths(dep_dir, current_dir) {
                Some(path) => path,
                None => continue, // Skip if we can't calculate relative path
            };

            // Generate reference path based on style preference
            let reference_path = match reference_path_style {
                "tsconfig" => {
                    // Point to tsconfig.json file directly
                    relative_path.join("tsconfig.json").display().to_string()
                }
                _ => {
                    // Default to relative directory path
                    relative_path.display().to_string()
                }
            };

            new_references.push(TsReference {
                path: Some(reference_path),
            });
        }
    }

    // Check if references need updating
    let current_paths: Vec<String> = config
        .references
        .iter()
        .filter_map(|r| r.path.as_ref())
        .cloned()
        .collect();

    let new_paths: Vec<String> = new_references
        .iter()
        .filter_map(|r| r.path.as_ref())
        .cloned()
        .collect();

    if current_paths == new_paths {
        return Ok(false); // No changes needed
    }

    // Update references
    config.references = new_references;

    // Write back to file with pretty formatting
    let updated_json = serde_json::to_string_pretty(&config)?;
    fs::write(tsconfig_path, updated_json)?;

    Ok(true) // Changes were made
}

/// Configuration options for the TypeScript plugin
#[derive(Debug, Deserialize, Default)]
pub struct TypeScriptPluginConfig {
    #[serde(default)]
    pub auto_project_references: bool,
    #[serde(default = "default_reference_path_style")]
    pub reference_path_style: String,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
}

fn default_reference_path_style() -> String {
    "relative".to_string()
}

/// Check if a TypeScript project should have auto project references enabled
pub fn should_auto_update_references(config_options: Option<&JsonValue>) -> bool {
    config_options
        .and_then(|v| serde_json::from_value::<TypeScriptPluginConfig>(v.clone()).ok())
        .map(|config| config.auto_project_references)
        .unwrap_or(false)
}

/// Update all TypeScript projects in a workspace with project references based on workspace dependencies detected by other plugins
pub fn update_workspace_project_references(
    workspace: &Workspace,
    config_options: Option<&JsonValue>,
) -> anyhow::Result<Vec<String>> {
    let config = config_options
        .and_then(|v| serde_json::from_value::<TypeScriptPluginConfig>(v.clone()).ok())
        .unwrap_or_default();

    if !config.auto_project_references {
        return Ok(Vec::new());
    }

    let mut updated_projects = Vec::new();

    // Process all TypeScript projects (both explicit and inferred)
    let ts_projects: Vec<(&String, &PathBuf, &[String])> = workspace
        .inferred_projects
        .iter()
        .filter(|p| p.discovered_by == "typescript")
        .map(|p| (&p.name, &p.project_dir, p.workspace_dependencies.as_slice()))
        .collect();

    for (project_name, project_dir, dependencies) in ts_projects {
        let tsconfig_path = project_dir.join("tsconfig.json");

        if tsconfig_path.exists() && !dependencies.is_empty() {
            match update_project_references(
                &tsconfig_path,
                dependencies,
                workspace,
                &config.reference_path_style,
            ) {
                Ok(true) => {
                    updated_projects.push(format!(
                        "Updated project references for: {} ({})",
                        project_name,
                        tsconfig_path.display()
                    ));
                }
                Ok(false) => {
                    // No changes needed - references were already up to date
                }
                Err(e) => {
                    eprintln!(
                        "Failed to update project references for {}: {}",
                        project_name, e
                    );
                }
            }
        }
    }

    Ok(updated_projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_typescript_project() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("api");
        std::fs::create_dir_all(&project_dir).unwrap();

        let config = r#"
{
  "compilerOptions": {
    "composite": true
  },
  "references": [
    { "path": "../shared/tsconfig.json" },
    { "path": "../ui" }
  ]
}
"#;

        let message = process_tsconfig(&project_dir.join("tsconfig.json"), config)
            .expect("should return inferred project");

        assert_eq!(message.name, "api");
        assert_eq!(message.discovered_by, "typescript");
        assert_eq!(message.project_dir, project_dir.display().to_string());
        // TypeScript plugin doesn't detect workspace dependencies - that's the PNPM plugin's job
        assert_eq!(message.workspace_dependencies, Vec::<String>::new());
    }

    #[test]
    fn updates_project_references() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        // Create project structure
        let api_dir = workspace_root.join("api");
        let shared_dir = workspace_root.join("shared");
        let ui_dir = workspace_root.join("ui");

        fs::create_dir_all(&api_dir).unwrap();
        fs::create_dir_all(&shared_dir).unwrap();
        fs::create_dir_all(&ui_dir).unwrap();

        // Create initial tsconfig.json for api project
        let api_tsconfig = api_dir.join("tsconfig.json");
        fs::write(
            &api_tsconfig,
            r#"
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist"
  }
}
"#,
        )
        .unwrap();

        // Create mock workspace
        let workspace = Workspace {
            root: workspace_root.to_path_buf(),
            projects: vec![],
            inferred_projects: vec![
                InferredProject {
                    name: "shared".to_string(),
                    project_dir: shared_dir.clone(),
                    discovered_by: "typescript".to_string(),
                    workspace_dependencies: vec![],
                },
                InferredProject {
                    name: "ui".to_string(),
                    project_dir: ui_dir.clone(),
                    discovered_by: "typescript".to_string(),
                    workspace_dependencies: vec!["shared".to_string()],
                },
            ],
        };

        // Update project references
        let dependencies = vec!["shared".to_string()];
        let result =
            update_project_references(&api_tsconfig, &dependencies, &workspace, "relative");

        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true indicating changes were made

        // Verify the file was updated
        let updated_content = fs::read_to_string(&api_tsconfig).unwrap();
        let config: TsConfig = serde_json::from_str(&updated_content).unwrap();

        assert_eq!(config.references.len(), 1);
        assert_eq!(config.references[0].path, Some("../shared".to_string()));
    }

    #[test]
    fn updates_project_references_with_tsconfig_style() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        // Create project structure
        let api_dir = workspace_root.join("api");
        let shared_dir = workspace_root.join("shared");

        fs::create_dir_all(&api_dir).unwrap();
        fs::create_dir_all(&shared_dir).unwrap();

        // Create initial tsconfig.json
        let api_tsconfig = api_dir.join("tsconfig.json");
        fs::write(&api_tsconfig, r#"{"compilerOptions": {"composite": true}}"#).unwrap();

        // Create mock workspace
        let workspace = Workspace {
            root: workspace_root.to_path_buf(),
            projects: vec![],
            inferred_projects: vec![InferredProject {
                name: "shared".to_string(),
                project_dir: shared_dir.clone(),
                discovered_by: "typescript".to_string(),
                workspace_dependencies: vec![],
            }],
        };

        // Update project references with tsconfig style
        let dependencies = vec!["shared".to_string()];
        let result =
            update_project_references(&api_tsconfig, &dependencies, &workspace, "tsconfig");

        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify the file was updated with tsconfig.json path
        let updated_content = fs::read_to_string(&api_tsconfig).unwrap();
        let config: TsConfig = serde_json::from_str(&updated_content).unwrap();

        assert_eq!(config.references.len(), 1);
        assert_eq!(
            config.references[0].path,
            Some("../shared/tsconfig.json".to_string())
        );
    }

    #[test]
    fn configuration_parsing() {
        let config_json = json!({
            "auto_project_references": true,
            "reference_path_style": "tsconfig",
            "includes": ["**/*.ts"],
            "excludes": ["**/*.test.ts"]
        });

        let config: TypeScriptPluginConfig = serde_json::from_value(config_json).unwrap();

        assert!(config.auto_project_references);
        assert_eq!(config.reference_path_style, "tsconfig");
        assert_eq!(config.includes, vec!["**/*.ts"]);
        assert_eq!(config.excludes, vec!["**/*.test.ts"]);
    }

    #[test]
    fn should_auto_update_references_returns_correct_value() {
        // Test with auto_project_references enabled
        let config_enabled = json!({
            "auto_project_references": true
        });
        assert!(should_auto_update_references(Some(&config_enabled)));

        // Test with auto_project_references disabled
        let config_disabled = json!({
            "auto_project_references": false
        });
        assert!(!should_auto_update_references(Some(&config_disabled)));

        // Test with no config
        assert!(!should_auto_update_references(None));

        // Test with invalid config
        let invalid_config = json!({
            "invalid": "config"
        });
        assert!(!should_auto_update_references(Some(&invalid_config)));
    }
}
