use std::collections::HashSet;
use std::path::Path;

use marty_plugin_protocol::InferredProjectMessage;
use toml::Value;

pub fn include_path_globs() -> Vec<String> {
    vec![
        "**/Cargo.toml".to_string(),
        "**/*.rs".to_string(),
    ]
}

#[deprecated(note = "Use include_path_globs instead")]
pub fn ignore_path_globs() -> Vec<String> {
    vec![
        "**/target/**".to_string(),
        "**/.git/**".to_string(),
        "**/node_modules/**".to_string(),
    ]
}

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
