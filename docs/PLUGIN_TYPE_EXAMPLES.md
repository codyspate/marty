# Plugin Type Examples

This document provides practical examples of implementing each plugin type in Marty.

## Table of Contents

- [Primary Plugin Example: Go Modules](#primary-plugin-example-go-modules)
- [Supplemental Plugin Example: Prettier Configuration](#supplemental-plugin-example-prettier-configuration)
- [Hook Plugin Example: Pre-Commit Validation](#hook-plugin-example-pre-commit-validation)

---

## Primary Plugin Example: Go Modules

A Primary plugin that discovers Go projects by scanning for `go.mod` files and detecting workspace dependencies.

### Full Implementation

```rust
use std::collections::HashSet;
use std::path::Path;

use marty_plugin_protocol::{
    dylib::export_plugin,
    InferredProject,
    MartyPlugin,
    PluginType,
    Workspace,
    WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};

/// Main Go plugin struct
pub struct GoPlugin;

/// Workspace provider for Go projects  
pub struct GoWorkspaceProvider;

impl Default for GoPlugin {
    fn default() -> Self {
        Self
    }
}

impl GoPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl WorkspaceProvider for GoWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // Scan for go.mod files
        vec!["**/go.mod".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![
            "**/vendor/**".to_string(),
            "**/.git/**".to_string(),
            "**/node_modules/**".to_string(),
        ]
    }

    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        // Only process go.mod files
        if path.file_name()?.to_str()? != "go.mod" {
            return None;
        }

        let project_dir = path.parent()?.to_path_buf();
        let contents = std::fs::read_to_string(path).ok()?;
        
        // Parse go.mod to extract module name and dependencies
        let module_name = Self::extract_module_name(&contents)?;
        let workspace_dependencies = Self::extract_workspace_deps(&contents, workspace);

        Some(InferredProject {
            name: module_name,
            project_dir,
            discovered_by: "go".to_string(),
            workspace_dependencies,
        })
    }
}

impl GoWorkspaceProvider {
    fn extract_module_name(contents: &str) -> Option<String> {
        // Parse module directive: "module example.com/myproject"
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with("module ") {
                return line.strip_prefix("module ")
                    .map(|s| s.trim().to_string());
            }
        }
        None
    }

    fn extract_workspace_deps(contents: &str, workspace: &Workspace) -> Vec<String> {
        let mut deps = HashSet::new();
        let mut in_require = false;

        for line in contents.lines() {
            let line = line.trim();
            
            if line.starts_with("require (") {
                in_require = true;
                continue;
            }
            if in_require && line == ")" {
                in_require = false;
                continue;
            }

            if in_require || line.starts_with("require ") {
                // Extract dependency module name
                let dep = line
                    .strip_prefix("require ")
                    .unwrap_or(line)
                    .split_whitespace()
                    .next()
                    .unwrap_or("");

                // Check if this is a workspace project
                if Self::is_workspace_project(dep, workspace) {
                    deps.insert(dep.to_string());
                }
            }
        }

        deps.into_iter().collect()
    }

    fn is_workspace_project(module_name: &str, workspace: &Workspace) -> bool {
        workspace.projects.iter().any(|p| p.name == module_name)
            || workspace.inferred_projects.iter().any(|p| p.name == module_name)
    }
}

impl MartyPlugin for GoPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary  // This plugin discovers projects
    }

    fn name(&self) -> &str {
        "Go Modules Plugin"
    }

    fn key(&self) -> &str {
        "go"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &GoWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<JsonValue> {
        Some(json!({
            "type": "object",
            "properties": {
                "include_test_modules": {
                    "type": "boolean",
                    "description": "Include test-only modules in discovery",
                    "default": false
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin
export_plugin!(GoPlugin);
```

### Key Points for Primary Plugins

1. **Non-empty glob patterns**: `include_path_globs()` returns specific patterns
2. **Creates projects**: `on_file_found()` returns `Some(InferredProject)` when appropriate
3. **Detects workspace deps**: Only includes internal workspace projects, NOT external packages
4. **Declares type**: `plugin_type()` returns `PluginType::Primary`

---

## Supplemental Plugin Example: Prettier Configuration

A Supplemental plugin that generates `.prettierrc.json` files for projects based on workspace structure.

### Full Implementation

```rust
use std::path::{Path, PathBuf};
use std::fs;

use marty_plugin_protocol::{
    dylib::export_plugin,
    InferredProject,
    MartyPlugin,
    PluginType,
    Workspace,
    WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};

/// Main Prettier plugin struct
pub struct PrettierPlugin;

/// Workspace provider for Prettier (supplemental only)
pub struct PrettierWorkspaceProvider;

impl Default for PrettierPlugin {
    fn default() -> Self {
        Self
    }
}

impl PrettierPlugin {
    pub const fn new() -> Self {
        Self
    }
    
    /// Process all JavaScript/TypeScript projects and ensure they have Prettier config
    pub fn update_prettier_configs(workspace: &Workspace, base_config: &JsonValue) -> anyhow::Result<()> {
        for project in workspace.inferred_projects.iter() {
            // Check if this is a JS/TS project (has package.json or tsconfig.json)
            if Self::is_js_ts_project(&project.project_dir) {
                Self::ensure_prettier_config(&project.project_dir, base_config)?;
            }
        }
        Ok(())
    }
    
    fn is_js_ts_project(project_dir: &Path) -> bool {
        project_dir.join("package.json").exists()
            || project_dir.join("tsconfig.json").exists()
    }
    
    fn ensure_prettier_config(project_dir: &Path, base_config: &JsonValue) -> anyhow::Result<()> {
        let prettier_config_path = project_dir.join(".prettierrc.json");
        
        // Don't overwrite existing configs
        if prettier_config_path.exists() {
            return Ok(());
        }
        
        // Create project-specific config
        let config = json!({
            "printWidth": base_config.get("printWidth").unwrap_or(&json!(80)),
            "tabWidth": base_config.get("tabWidth").unwrap_or(&json!(2)),
            "semi": base_config.get("semi").unwrap_or(&json!(true)),
            "singleQuote": base_config.get("singleQuote").unwrap_or(&json!(false)),
            "trailingComma": base_config.get("trailingComma").unwrap_or(&json!("es5")),
        });
        
        fs::write(
            prettier_config_path,
            serde_json::to_string_pretty(&config)?
        )?;
        
        Ok(())
    }
}

impl WorkspaceProvider for PrettierWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // Supplemental plugins don't discover projects
        vec![]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![]
    }

    fn on_file_found(&self, _workspace: &Workspace, _path: &Path) -> Option<InferredProject> {
        // Supplemental plugins never create projects
        None
    }
}

impl MartyPlugin for PrettierPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Supplemental  // This plugin enhances existing projects
    }

    fn name(&self) -> &str {
        "Prettier Configuration Plugin"
    }

    fn key(&self) -> &str {
        "prettier"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &PrettierWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<JsonValue> {
        Some(json!({
            "type": "object",
            "properties": {
                "printWidth": {
                    "type": "number",
                    "description": "Line length for formatting",
                    "default": 80
                },
                "tabWidth": {
                    "type": "number",
                    "description": "Number of spaces per indentation level",
                    "default": 2
                },
                "semi": {
                    "type": "boolean",
                    "description": "Add semicolons at ends of statements",
                    "default": true
                },
                "singleQuote": {
                    "type": "boolean",
                    "description": "Use single quotes instead of double quotes",
                    "default": false
                },
                "trailingComma": {
                    "type": "string",
                    "enum": ["none", "es5", "all"],
                    "description": "Print trailing commas",
                    "default": "es5"
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin
export_plugin!(PrettierPlugin);
```

### Key Points for Supplemental Plugins

1. **Empty glob patterns**: `include_path_globs()` returns `vec![]`
2. **Never creates projects**: `on_file_found()` always returns `None`
3. **Processes existing projects**: Uses separate methods like `update_prettier_configs()`
4. **Declares type**: `plugin_type()` returns `PluginType::Supplemental`
5. **Works with any Primary plugin**: Doesn't care which plugin discovered the projects

---

## Hook Plugin Example: Pre-Commit Validation

A Hook plugin that validates workspace state before commits (future feature - API not yet stable).

### Conceptual Implementation

```rust
use marty_plugin_protocol::{
    dylib::export_plugin,
    InferredProject,
    MartyPlugin,
    PluginType,
    Workspace,
    WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};
use std::path::Path;

/// Main pre-commit plugin struct
pub struct PreCommitPlugin;

/// Workspace provider for pre-commit (hook only)
pub struct PreCommitWorkspaceProvider;

impl Default for PreCommitPlugin {
    fn default() -> Self {
        Self
    }
}

impl PreCommitPlugin {
    pub const fn new() -> Self {
        Self
    }
    
    // NOTE: Lifecycle hook methods are not yet implemented in marty_plugin_protocol
    // This is a conceptual example of what they might look like:
    
    /// Called before a git commit
    pub fn on_pre_commit(_workspace: &Workspace) -> anyhow::Result<()> {
        // Validate that all projects build
        // Check for uncommitted dependencies
        // Run linters and formatters
        // etc.
        Ok(())
    }
    
    /// Called after a successful build
    pub fn on_post_build(_workspace: &Workspace) -> anyhow::Result<()> {
        // Run tests
        // Generate documentation
        // Update change logs
        // etc.
        Ok(())
    }
}

impl WorkspaceProvider for PreCommitWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // Hook plugins don't discover projects
        vec![]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec![]
    }

    fn on_file_found(&self, _workspace: &Workspace, _path: &Path) -> Option<InferredProject> {
        // Hook plugins never create projects
        None
    }
}

impl MartyPlugin for PreCommitPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Hook  // This plugin runs at lifecycle points
    }

    fn name(&self) -> &str {
        "Pre-Commit Validation Plugin"
    }

    fn key(&self) -> &str {
        "pre-commit"
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &PreCommitWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<JsonValue> {
        Some(json!({
            "type": "object",
            "properties": {
                "run_on_commit": {
                    "type": "boolean",
                    "description": "Enable pre-commit validation",
                    "default": true
                },
                "run_tests": {
                    "type": "boolean",
                    "description": "Run tests before committing",
                    "default": false
                },
                "format_code": {
                    "type": "boolean",
                    "description": "Auto-format code before committing",
                    "default": true
                }
            },
            "additionalProperties": false
        }))
    }
}

// Export the plugin
export_plugin!(PreCommitPlugin);
```

### Key Points for Hook Plugins

1. **Empty glob patterns**: `include_path_globs()` returns `vec![]`
2. **Never creates projects**: `on_file_found()` always returns `None`
3. **Lifecycle methods**: Implements hook methods (API not yet defined)
4. **Declares type**: `plugin_type()` returns `PluginType::Hook`
5. **Event-driven**: Reacts to workspace events rather than scanning

**Note**: Hook plugins are a planned feature. The lifecycle hook API is not yet implemented.

---

## Comparison Table

| Aspect | Primary | Supplemental | Hook |
|--------|---------|--------------|------|
| **Discovers Projects** | ✅ Yes | ❌ No | ❌ No |
| **include_path_globs()** | Non-empty | Empty `vec![]` | Empty `vec![]` |
| **on_file_found()** | Can return `Some()` | Always `None` | Always `None` |
| **Primary Purpose** | Find and create projects | Enhance existing projects | Run actions at events |
| **Examples** | PNPM, Cargo, Go | TypeScript, Prettier | Pre-commit, Linters |
| **Can Coexist** | Multiple allowed | Multiple allowed | Multiple allowed |
| **Status** | ✅ Stable | ✅ Stable | ⚠️ Planned |

---

## Best Practices

### For All Plugin Types

1. **Declare type explicitly**: Always implement `plugin_type()` clearly
2. **Match behavior to type**: Ensure your plugin actually does what its type says
3. **Document purpose**: Explain why you chose this type
4. **Test thoroughly**: Verify your plugin behaves correctly for its type

### Type-Specific Guidelines

**Primary**:
- Focus on accurate workspace dependency detection
- Only include internal workspace projects in dependencies
- Be efficient with file I/O
- Use specific glob patterns

**Supplemental**:
- Never call `on_file_found()` with anything but `None`
- Process all relevant projects, regardless of discoverer
- Don't assume project structure beyond what's guaranteed
- Provide graceful fallbacks

**Hook**:
- Keep hooks fast and efficient
- Don't block critical operations unnecessarily
- Provide configuration to disable hooks
- Log actions clearly

---

## Testing Your Plugin Type

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.plugin_type(), PluginType::Primary);
    }

    #[test]
    fn test_primary_returns_globs() {
        let provider = MyWorkspaceProvider;
        let globs = provider.include_path_globs();
        assert!(!globs.is_empty(), "Primary plugins must return globs");
    }

    #[test]
    fn test_supplemental_returns_no_globs() {
        let provider = MyWorkspaceProvider;
        let globs = provider.include_path_globs();
        assert!(globs.is_empty(), "Supplemental plugins must not return globs");
    }

    #[test]
    fn test_supplemental_never_discovers() {
        let provider = MyWorkspaceProvider;
        let workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects: vec![],
            inferred_projects: vec![],
        };
        let path = PathBuf::from("/workspace/project/file.json");
        let result = provider.on_file_found(&workspace, &path);
        assert!(result.is_none(), "Supplemental plugins must not discover projects");
    }
}
```

---

## See Also

- [Plugin Types Documentation](./PLUGIN_TYPES.md)
- [Plugin Developer Guide](./PLUGIN_DEVELOPER_GUIDE.md)
- [Implementation Summary](./PLUGIN_TYPE_IMPLEMENTATION_SUMMARY.md)
