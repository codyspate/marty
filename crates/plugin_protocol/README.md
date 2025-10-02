# Marty Plugin Protocol

[![Crates.io](https://img.shields.io/crates/v/marty_plugin_protocol.svg)](https://crates.io/crates/marty_plugin_protocol)
[![Documentation](https://docs.rs/marty_plugin_protocol/badge.svg)](https://docs.rs/marty_plugin_protocol)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/codyspate/marty#license)

The official SDK for creating [Marty](https://github.com/codyspate/marty) workspace plugins as dynamic libraries. This crate provides everything you need to build plugins that automatically discover and manage projects within monorepos.

## 🚀 Quick Start

### 1. Create a New Plugin

```toml
# Cargo.toml
[package]
name = "marty-plugin-myframework"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for dynamic library

[dependencies]
marty_plugin_protocol = "0.2"
serde_json = "1.0"
```

### 2. Implement Your Plugin

```rust
use marty_plugin_protocol::{
    dylib::export_plugin, InferredProject, MartyPlugin, Workspace, WorkspaceProvider,
};
use serde_json::{json, Value as JsonValue};
use std::path::Path;

// Define your plugin
pub struct MyFrameworkPlugin;

impl MyFrameworkPlugin {
    pub const fn new() -> Self { Self }
}

impl Default for MyFrameworkPlugin {
    fn default() -> Self { Self::new() }
}

// Define project detection logic
pub struct MyFrameworkWorkspaceProvider;

impl WorkspaceProvider for MyFrameworkWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/my-framework.json".to_string()]
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        vec!["**/node_modules/**".to_string()]
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        if path.file_name()?.to_str()? != "my-framework.json" {
            return None;
        }

        let project_dir = path.parent()?.to_path_buf();
        let name = project_dir.file_name()?.to_str()?.to_string();

        Some(InferredProject {
            name,
            project_dir,
            discovered_by: "my-framework".to_string(),
            workspace_dependencies: Vec::new(), // Parse from config file
        })
    }
}

// Connect plugin to workspace provider
impl MartyPlugin for MyFrameworkPlugin {
    fn name(&self) -> &str { "My Framework Plugin" }
    fn key(&self) -> &str { "my-framework" }
    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        &MyFrameworkWorkspaceProvider
    }
}

// Export for dynamic loading
export_plugin!(MyFrameworkPlugin);
```

### 3. Build and Install

```bash
# Build the plugin
cargo build --lib --release

# Install to Marty's plugin directory
cp target/release/libmarty_plugin_myframework.so ~/.marty/plugins/
```

## 🧠 Core Concepts

### Plugin Discovery Process

1. **Scanning**: Marty walks the workspace using your `include_path_globs()` patterns
2. **Filtering**: Files matching `exclude_path_globs()` are skipped
3. **Detection**: `on_file_found()` is called for each matching file
4. **Project Creation**: Valid projects become `InferredProject` instances

### Workspace Dependencies ⚠️

**Critical**: `workspace_dependencies` represents dependencies between projects *within the same workspace*, NOT external packages.

```rust
// ✅ CORRECT: Internal workspace projects
vec![
    "shared-utils".to_string(),    // Another project in workspace
    "common-types".to_string(),    // Internal dependency
]

// ❌ INCORRECT: External packages
vec![
    "serde".to_string(),          // External crate
    "lodash".to_string(),         // External npm package
]
```

This is used for:
- **Build ordering**: Dependencies built first
- **Change impact**: Which projects are affected by changes  
- **Task orchestration**: Proper execution order across projects

## 📚 Examples by Language

### Rust (Cargo.toml)
```toml
[dependencies]
shared-utils = { path = "../shared-utils" }  # ✅ Include "shared-utils"
serde = "1.0"                                # ❌ External - ignore
```

### JavaScript/TypeScript (package.json)
```json
{
  "dependencies": {
    "@myorg/shared": "workspace:*",          // ✅ Include "shared"
    "@myorg/utils": "file:../utils",         // ✅ Include "utils"
    "lodash": "^4.17.21"                     // ❌ External - ignore
  }
}
```

### Python (pyproject.toml)
```toml
[tool.poetry.dependencies]
shared-lib = { path = "../shared-lib" }     # ✅ Include "shared-lib"
requests = "^2.25.0"                        # ❌ External - ignore
```

## 🔌 Plugin Architecture

### Core Traits

- **`MartyPlugin`**: Main plugin interface with metadata and configuration
- **`WorkspaceProvider`**: Project discovery and analysis logic

### Key Data Structures

- **`InferredProject`**: Represents a discovered project
- **`Workspace`**: Context passed to plugins (root path, existing projects)
- **`Project`**: Explicit projects with marty.yml files

### Dynamic Library Interface

The `export_plugin!` macro generates C ABI functions for cross-language compatibility:
- `plugin_name()` - Plugin display name
- `plugin_key()` - Unique identifier
- `plugin_on_file_found()` - Project detection
- `plugin_cleanup_string()` - Memory management

## 🧪 Testing Your Plugin

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_project_detection() {
        let provider = MyFrameworkWorkspaceProvider;
        let workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects: vec![],
            inferred_projects: vec![],
        };
        
        let path = PathBuf::from("/workspace/project/my-framework.json");
        let result = provider.on_file_found(&workspace, &path);
        
        assert!(result.is_some());
        let project = result.unwrap();
        assert_eq!(project.name, "project");
        assert_eq!(project.discovered_by, "my-framework");
    }
}
```

## 🚀 Performance Tips

- **Specific patterns**: Use precise glob patterns to minimize file system operations
- **Fast rejection**: Check filenames before reading file contents
- **Efficient parsing**: Use streaming parsers for large files
- **Smart exclusions**: Add common build directories to `exclude_path_globs()`

## 🛠️ Migration from WASM

If you're migrating from WASM plugins:

### Build Changes
```toml
# OLD: WASM target
# [lib]
# crate-type = ["cdylib"]
# [dependencies]
# serde = { version = "1.0", default-features = false }

# NEW: Dynamic library
[lib]
crate-type = ["cdylib"]
[dependencies]
serde_json = "1.0"  # Full crate ecosystem available!
```

### Code Changes
```rust
// OLD: Manual WASM exports
// #[no_mangle]
// pub extern "C" fn _start() { /* ... */ }

// NEW: Simple macro
export_plugin!(MyPlugin);
```

### Benefits
- ✅ **Faster execution** - Native vs WASM interpretation
- ✅ **Better debugging** - Standard debugging tools work
- ✅ **Full ecosystem** - Any Rust crate can be used
- ✅ **Simpler deployment** - Standard shared libraries

## 📖 Documentation

- [**API Documentation**](https://docs.rs/marty_plugin_protocol) - Complete API reference
- [**Plugin Guide**](https://github.com/codyspate/marty/tree/main/examples/example_plugin) - Detailed examples
- [**Marty Documentation**](https://github.com/codyspate/marty) - Main project docs

## 🐛 Troubleshooting

### Common Issues

**Plugin not detected:**
- Ensure `crate-type = ["cdylib"]` in Cargo.toml
- Check plugin is in `~/.marty/plugins/` directory
- Verify `export_plugin!(YourPlugin)` is called

**Projects not discovered:**
- Check `include_path_globs()` patterns match your files
- Verify `on_file_found()` returns `Some(InferredProject)` for valid projects
- Test patterns don't conflict with excludes

**Build errors:**
- Plugin struct must have `pub const fn new() -> Self`
- Must implement `MartyPlugin` trait
- Import `export_plugin` from `marty_plugin_protocol::dylib`

## 🤝 Contributing

Contributions are welcome! Please see the [main repository](https://github.com/codyspate/marty) for contributing guidelines.

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

---

**Built by the Marty team** 🚀 [GitHub](https://github.com/codyspate/marty) | [Documentation](https://docs.rs/marty_plugin_protocol)