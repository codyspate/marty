# Plugin Types in Marty

## Overview

Marty plugins now have explicit types that define their role and capabilities. This makes plugin development more structured, enables better type safety, and allows Marty to optimize plugin execution.

## The Three Plugin Types

### 1. Primary Plugins 

**Purpose**: Discover projects and their workspace dependencies

Primary plugins are the source of truth for project structure in a monorepo. They scan the workspace for project configuration files, extract metadata, and detect dependencies between workspace projects.

**Responsibilities**:
- Scan workspace for project files (e.g., `package.json`, `Cargo.toml`, `pyproject.toml`)
- Extract project metadata (name, version, location)
- Detect workspace dependencies (NOT external dependencies)
- Create `InferredProject` instances for each discovered project

**Characteristics**:
- `include_path_globs()` returns non-empty glob patterns
- `on_file_found()` can return `Some(InferredProject)`
- Multiple Primary plugins can coexist (e.g., Cargo + PNPM for polyglot repos)

**Examples**:
- **PNPM Plugin**: Discovers JavaScript/TypeScript projects via `package.json`
- **Cargo Plugin**: Discovers Rust projects via `Cargo.toml`
- **NPM Plugin**: Discovers JavaScript/TypeScript projects via `package.json`
- **Python Plugin**: Discovers Python projects via `pyproject.toml` or `setup.py`

**Implementation Example**:
```rust
impl MartyPlugin for PnpmPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary
    }
    
    // ... other methods
}

impl WorkspaceProvider for PnpmWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/package.json".to_string()]
    }
    
    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        // Parse package.json, detect workspace dependencies
        // Return Some(InferredProject { ... })
    }
}
```

### 2. Supplemental Plugins

**Purpose**: Enhance existing projects without discovering new ones

Supplemental plugins add language/framework-specific functionality to projects that have already been discovered by Primary plugins. They NEVER create new projects.

**Responsibilities**:
- Generate or update configuration files
- Add framework-specific tooling
- Provide additional project enhancements
- Process ALL projects that match their criteria, regardless of which plugin discovered them

**Characteristics**:
- `include_path_globs()` typically returns empty `vec![]`
- `on_file_found()` ALWAYS returns `None`
- Use lifecycle hooks or direct project processing instead
- Work with projects discovered by ANY Primary plugin

**Examples**:
- **TypeScript Plugin**: Adds project references to `tsconfig.json` for projects discovered by PNPM/NPM
- **ESLint Plugin** (hypothetical): Generates `.eslintrc` files based on workspace structure
- **Docker Plugin** (hypothetical): Creates `docker-compose.yml` entries for each service project

**Implementation Example**:
```rust
impl MartyPlugin for TypeScriptPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Supplemental
    }
    
    // ... other methods
}

impl WorkspaceProvider for TypeScriptWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec![]  // Don't scan for files - we're supplemental only
    }
    
    fn on_file_found(&self, _workspace: &Workspace, _path: &Path) -> Option<InferredProject> {
        None  // Never create projects
    }
}
```

**Key Design Pattern**: Supplemental plugins should process projects in a separate phase, typically through a method like `update_workspace_project_references()` that operates on the already-discovered workspace.

### 3. Hook Plugins (Future Feature)

**Purpose**: Execute actions at lifecycle points without discovering projects

Hook plugins run commands or scripts at specific points in the workspace lifecycle. This feature is planned but not yet implemented.

**Planned Responsibilities**:
- Run pre-commit or post-build checks
- Validate workspace state
- Perform maintenance tasks
- Integrate with external tools

**Characteristics** (when implemented):
- `include_path_globs()` returns empty `vec![]`
- `on_file_found()` returns `None`
- Implement lifecycle hook methods (to be defined)

**Examples** (planned):
- **Pre-commit Plugin**: Run linters and formatters before commits
- **CI/CD Plugin**: Trigger builds and deployments
- **Notification Plugin**: Send alerts on workspace changes
- **Validation Plugin**: Check for policy compliance

## Migration Guide for Plugin Developers

### For Existing Plugins

All existing plugins need to add the `plugin_type()` method to their `MartyPlugin` implementation:

```rust
impl MartyPlugin for MyPlugin {
    // NEW: Add this method
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary  // or Supplemental or Hook
    }
    
    // Existing methods...
    fn name(&self) -> &str { ... }
    fn key(&self) -> &str { ... }
    fn workspace_provider(&self) -> &dyn WorkspaceProvider { ... }
    fn configuration_options(&self) -> Option<JsonValue> { ... }
}
```

### Decision Tree: Choosing Your Plugin Type

```text
┌────────────────────────────────────────────────────────────┐
│ Does your plugin discover new projects?                    │
│ (scan for config files, extract names & dependencies)      │
└──────────────┬────────────────────────────┬────────────────┘
               │                            │
              YES                          NO
               │                            │
               ▼                            ▼
       ┌───────────────┐          ┌────────────────────────┐
       │ Primary       │          │ Does it enhance        │
       │               │          │ existing projects?     │
       │ Example:      │          └──────┬──────────┬──────┘
       │ - PNPM        │                 │          │
       │ - Cargo       │                YES        NO
       │ - NPM         │                 │          │
       └───────────────┘                 ▼          ▼
                               ┌──────────────┐  ┌─────────┐
                               │ Supplemental │  │  Hook   │
                               │              │  │         │
                               │ Example:     │  │ Example:│
                               │ - TypeScript │  │ - Hooks │
                               │              │  │ - Tasks │
                               └──────────────┘  └─────────┘
```

### Updated Imports

Plugins need to import `PluginType`:

```rust
use marty_plugin_protocol::{
    dylib::export_plugin,
    InferredProject,
    MartyPlugin,
    PluginType,  // NEW: Import PluginType
    Workspace,
    WorkspaceProvider,
};
```

## Benefits of Explicit Plugin Types

### 1. Type Safety
- Compile-time guarantees that plugins declare their behavior
- Prevents accidental misuse of plugin APIs
- Makes plugin capabilities immediately clear

### 2. Performance Optimization
- Marty can skip file scanning for Supplemental and Hook plugins
- Better parallelization based on plugin types
- Reduced I/O operations

### 3. Better Developer Experience
- Clear documentation of plugin purpose
- Easier to understand plugin architecture
- Self-documenting code

### 4. Validation
- Marty can warn if a Supplemental plugin returns projects
- Runtime checks ensure plugins behave as declared
- Better error messages for misconfigured plugins

### 5. Future Extensibility
- Foundation for lifecycle hooks and events
- Enables plugin dependency ordering
- Supports more complex plugin ecosystems

## Technical Details

### FFI Interface

The plugin type is exposed through the FFI boundary as a simple `u8`:

```rust
#[no_mangle]
pub extern "C" fn plugin_type() -> u8 {
    match PLUGIN.plugin_type() {
        PluginType::Primary => 0,
        PluginType::Supplemental => 1,
        PluginType::Hook => 2,
    }
}
```

### Enum Definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// Plugin discovers projects and their workspace dependencies.
    Primary,
    /// Plugin enhances existing projects without discovering new ones.
    Supplemental,
    /// Plugin executes actions at lifecycle hooks without discovering projects.
    Hook,
}
```

### Helper Methods

```rust
impl PluginType {
    /// Returns whether this plugin type is expected to discover projects.
    pub const fn discovers_projects(&self) -> bool {
        matches!(self, Self::Primary)
    }

    /// Returns a human-readable description of this plugin type.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Primary => "Discovers projects and workspace dependencies",
            Self::Supplemental => "Enhances existing projects without discovering new ones",
            Self::Hook => "Executes actions at lifecycle hooks",
        }
    }
}
```

## Examples

### Converting a Discovery Plugin to Supplemental

**Before** (TypeScript plugin discovering projects):
```rust
impl WorkspaceProvider for TypeScriptWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec!["**/tsconfig.json".to_string()]
    }
    
    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        // Create InferredProject from tsconfig.json
        Some(InferredProject { ... })
    }
}

impl MartyPlugin for TypeScriptPlugin {
    // No plugin_type method
    fn name(&self) -> &str { "TypeScript Plugin" }
    // ...
}
```

**After** (TypeScript plugin as supplemental):
```rust
impl WorkspaceProvider for TypeScriptWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        vec![]  // Don't discover projects
    }
    
    fn on_file_found(&self, _workspace: &Workspace, _path: &Path) -> Option<InferredProject> {
        None  // Never create projects
    }
}

impl MartyPlugin for TypeScriptPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Supplemental  // Explicit type declaration
    }
    
    fn name(&self) -> &str { "TypeScript Plugin" }
    // ...
}
```

## FAQ

**Q: Can a plugin be both Primary and Supplemental?**
A: No. Each plugin must choose a single type that best describes its primary responsibility. If you need both behaviors, create two separate plugins.

**Q: What happens if I declare Supplemental but return projects?**
A: Marty will log a warning. The plugin will still function, but it's violating the contract. Future versions may enforce this more strictly.

**Q: Can I change my plugin's type later?**
A: Changing from Primary to Supplemental (or vice versa) is a breaking change. Users may depend on your plugin's discovery behavior. Increment your major version and document the change clearly.

**Q: Why not use Rust features instead of an enum?**
A: An enum is simpler, more explicit, and easier to serialize across the FFI boundary. It also makes runtime introspection straightforward.

**Q: When should I use Hook type?**
A: Hook plugins are not yet fully implemented. Wait for the lifecycle hooks API to be defined before using this type.

## See Also

- [Plugin Developer Guide](./PLUGIN_DEVELOPER_GUIDE.md)
- [Plugin Architecture Documentation](./TYPESCRIPT_PLUGIN_ARCHITECTURE.md)
- [Plugin Protocol API Reference](../crates/plugin_protocol/src/lib.rs)
