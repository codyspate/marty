use std::collections::HashSet;
use std::path::Path;

use marty_plugin_protocol::InferredProjectMessage;
use serde::Deserialize;

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
