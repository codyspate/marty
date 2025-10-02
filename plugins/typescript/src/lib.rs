use std::path::{Path, PathBuf};

use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, InferredProjectMessage, MartyPlugin, Workspace,
    WorkspaceProvider,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Deserialize, Default)]
struct TsConfig {
    #[serde(default)]
    references: Vec<TsReference>,
}

#[derive(Debug, Deserialize, Default)]
struct TsReference {
    #[serde(default)]
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

    let config: TsConfig = serde_json::from_str(contents).unwrap_or_default();
    let project_dir = manifest_path.parent()?.to_path_buf();

    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())?;

    let dependencies = config
        .references
        .into_iter()
        .filter_map(|reference| reference.path)
        .filter_map(|raw_path| {
            let path_buf = PathBuf::from(&raw_path);
            let name_source: Option<&Path> = if path_buf
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.eq_ignore_ascii_case("tsconfig.json"))
                .unwrap_or(false)
            {
                path_buf.parent()
            } else {
                Some(path_buf.as_path())
            };

            name_source
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
        })
        .collect::<Vec<_>>();

    Some(InferredProjectMessage::new(
        name,
        project_dir.display().to_string(),
        "typescript",
        dedupe_sorted(dependencies),
    ))
}

fn dedupe_sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort_unstable();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extracts_reference_names() {
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
        assert_eq!(message.workspace_dependencies, vec!["shared", "ui"]);
    }
}
