# MartyPlugin Trait Migration Guide

## Overview

The Marty plugin system has been updated to use the new `MartyPlugin` trait, which provides a cleaner and more comprehensive interface for plugin developers. This change consolidates all plugin functionality into a single trait and provides better plugin metadata support.

## What Changed

### Before: WorkspaceProvider Only
```rust
use marty_plugin_protocol::{WorkspaceProvider, Workspace, InferredProject};

struct MyPlugin;

impl WorkspaceProvider for MyPlugin {
    fn include_path_globs(&self) -> Vec<String> { /* ... */ }
    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> { /* ... */ }
}
```

### After: MartyPlugin + WorkspaceProvider
```rust
use marty_plugin_protocol::{MartyPlugin, WorkspaceProvider, Workspace, InferredProject};

struct MyPlugin;
struct MyWorkspaceProvider;

impl WorkspaceProvider for MyWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> { /* ... */ }
    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> { /* ... */ }
}

impl MartyPlugin for MyPlugin {
    fn name(&self) -> &str { "My Plugin" }
    fn key(&self) -> &str { "my-plugin" }
    fn workspace_provider(&self) -> &dyn WorkspaceProvider { &MyWorkspaceProvider }
    fn configuration_options(&self) -> Option<serde_json::Value> { None }
}
```

## Benefits of MartyPlugin Trait

### 1. **Clear Plugin Identity**
- **`name()`**: Human-readable plugin name for display in UI/logs
- **`key()`**: Machine-readable identifier (no whitespace) for configuration and internal mapping

### 2. **Configuration Schema**
- **`configuration_options()`**: JSON schema describing valid plugin configuration options
- Enables validation of plugin configurations in workspace files

### 3. **Better Architecture**
- Separates plugin metadata from workspace scanning logic
- Allows for future extensibility without breaking changes
- Provides consistent interface across all plugins

## Implementation Guide

### Step 1: Create Your Workspace Provider
```rust
use marty_plugin_protocol::{WorkspaceProvider, Workspace, InferredProject};
use std::path::Path;

struct MyWorkspaceProvider;

impl WorkspaceProvider for MyWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/my-config.json".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec!["**/node_modules/**".to_string()]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        if path.file_name()?.to_str()? == "my-config.json" {
            let project_dir = path.parent()?.to_path_buf();
            let name = project_dir.file_name()?.to_str()?.to_string();
            
            Some(InferredProject {
                name,
                project_dir,
                discovered_by: "my-plugin".to_string(),
                workspace_dependencies: Vec::new(),
            })
        } else {
            None
        }
    }
}
```

### Step 2: Implement MartyPlugin
```rust
use marty_plugin_protocol::MartyPlugin;
use serde_json::{json, Value};

struct MyPlugin;

impl MartyPlugin for MyPlugin {
    fn name(&self) -> &str {
        "My Custom Plugin"  // Displayed in logs and UI
    }

    fn key(&self) -> &str {
        "my-plugin"  // Used in configuration, must be unique and no whitespace
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &MyWorkspaceProvider
    }

    fn configuration_options(&self) -> Option<Value> {
        Some(json!({
            "properties": {
                "build_command": {
                    "type": "string",
                    "description": "Custom build command",
                    "default": "build"
                },
                "enable_tests": {
                    "type": "boolean",
                    "description": "Enable test discovery",
                    "default": true
                }
            },
            "additionalProperties": false
        }))
    }
}
```

### Step 3: WASM Export (for compiled plugins)
```rust
#[no_mangle]
pub extern "C" fn _start() {
    let plugin = MyPlugin;
    
    // Handle WASM commands using plugin methods
    // Implementation depends on your WASM runtime interface
}
```

## Plugin Configuration

With the new `MartyPlugin` trait, users can configure plugins in their `workspace.yml`:

```yaml
plugins:
  - name: my-plugin
    url: "https://example.com/my-plugin.wasm"
    options:
      build_command: "custom-build"
      enable_tests: false
```

The `configuration_options()` method provides a JSON schema that validates these options.

## Key Features

### Plugin Key Usage
The plugin key is used for:
- **Configuration mapping**: Identifying plugins in workspace.yml
- **Internal references**: Non-whitespace string for reliable identification
- **Deduplication**: Ensuring the same plugin isn't loaded multiple times

### Plugin Name Display
The plugin name is used for:
- **User interface**: Showing friendly names in CLI output
- **Logging**: Clear identification in error messages and debug output
- **Documentation**: Human-readable plugin identification

### Configuration Validation
The configuration options provide:
- **Schema validation**: Ensures plugin options are valid
- **Documentation**: Self-documenting plugin capabilities
- **IDE support**: Enables autocompletion and validation in editors

## Migration Checklist

- [ ] Separate `WorkspaceProvider` implementation from main plugin struct
- [ ] Implement `MartyPlugin` trait with all required methods
- [ ] Define `name()` - human-readable plugin name
- [ ] Define `key()` - machine-readable identifier (no whitespace)
- [ ] Return workspace provider instance from `workspace_provider()`
- [ ] Optionally define configuration schema in `configuration_options()`
- [ ] Update WASM exports to use new trait methods
- [ ] Test plugin loading and configuration validation

## Example Projects

See the following examples for complete implementations:
- `examples/example_plugin/` - Python requirements.txt plugin
- `plugins/cargo/` - Cargo.toml plugin (updated)
- `plugins/typescript/` - TypeScript plugin (updated)
- `plugins/pnpm/` - PNPM plugin (updated)