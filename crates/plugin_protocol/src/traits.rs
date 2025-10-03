//! Core traits for implementing Marty plugins.
//!
//! This module defines the two main traits that plugins must implement:
//! - [`MartyPlugin`] - Main plugin interface with metadata and configuration
//! - [`WorkspaceProvider`] - Project discovery and scanning logic

use crate::types::{InferredProject, PluginType, Workspace};
use serde_json::Value as JsonValue;
use std::path::Path;

/// The workspace provider trait that plugins implement to discover projects.
///
/// **Purpose**: This trait defines how your plugin scans the workspace and detects projects.
/// It's the core of the plugin's project discovery logic.
///
/// **Implementation Pattern**: Most plugins create a separate struct for the workspace
/// provider and return a reference to it from the `MartyPlugin::workspace_provider()` method.
///
/// # Example
///
/// ```rust
/// # use marty_plugin_protocol::{WorkspaceProvider, Workspace, InferredProject};
/// # use std::path::Path;
/// pub struct MyWorkspaceProvider;
///
/// impl WorkspaceProvider for MyWorkspaceProvider {
///     fn include_path_globs(&self) -> Vec<String> {
///         vec!["**/my-config.json".to_string()]
///     }
///
///     fn exclude_path_globs(&self) -> Vec<String> {
///         vec!["**/node_modules/**".to_string()]
///     }
///
///     fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
///         // Project detection logic here
///         None
///     }
/// }
/// ```
pub trait WorkspaceProvider {
    /// Return glob patterns for paths to include when scanning the workspace.
    ///
    /// **Purpose**: Tell Marty which files your plugin is interested in processing.
    /// These patterns are used to filter the filesystem walk before calling `on_file_found()`.
    ///
    /// **Performance Tip**: Be as specific as possible to avoid unnecessary file system operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::WorkspaceProvider;
    /// # struct MyProvider;
    /// # impl WorkspaceProvider for MyProvider {
    /// fn include_path_globs(&self) -> Vec<String> {
    ///     vec![
    ///         // Specific filenames (most efficient)
    ///         "**/Cargo.toml".to_string(),
    ///         "**/package.json".to_string(),
    ///         
    ///         // File patterns  
    ///         "**/*.csproj".to_string(),
    ///         "**/requirements*.txt".to_string(),
    ///         
    ///         // Directory patterns
    ///         "**/src/**/*.rs".to_string(),
    ///     ]
    /// }
    /// # fn on_file_found(&self, _: &marty_plugin_protocol::Workspace, _: &std::path::Path) -> Option<marty_plugin_protocol::InferredProject> { None }
    /// # }
    /// ```
    ///
    /// **Common Patterns**:
    /// - `**/filename.ext` - Specific filename in any directory
    /// - `**/*.ext` - All files with specific extension
    /// - `**/dir/**` - All files in directories named "dir"
    /// - `path/*/file` - File in immediate subdirectories of path
    fn include_path_globs(&self) -> Vec<String>;

    /// Return glob patterns for paths to exclude when scanning the workspace.
    ///
    /// **Purpose**: Improve performance by skipping directories/files that won't contain projects.
    /// These patterns are applied *in addition* to Marty's built-in excludes.
    ///
    /// **Built-in Excludes** (automatically applied):
    /// - `**/.git/**` - Git repository data
    /// - `**/target/**` - Rust build outputs  
    /// - `**/node_modules/**` - Node.js dependencies
    /// - `**/.marty/**` - Marty configuration directory
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::WorkspaceProvider;
    /// # struct MyProvider;
    /// # impl WorkspaceProvider for MyProvider {
    /// # fn include_path_globs(&self) -> Vec<String> { vec![] }
    /// fn exclude_path_globs(&self) -> Vec<String> {
    ///     vec![
    ///         // Build output directories
    ///         "**/dist/**".to_string(),
    ///         "**/build/**".to_string(),
    ///         "**/.next/**".to_string(),
    ///         
    ///         // Cache directories
    ///         "**/.cache/**".to_string(),
    ///         "**/tmp/**".to_string(),
    ///         
    ///         // IDE/Editor files
    ///         "**/.vscode/**".to_string(),
    ///         "**/*.log".to_string(),
    ///     ]
    /// }
    /// # fn on_file_found(&self, _: &marty_plugin_protocol::Workspace, _: &std::path::Path) -> Option<marty_plugin_protocol::InferredProject> { None }
    /// # }
    /// ```
    fn exclude_path_globs(&self) -> Vec<String> {
        Vec::new() // Default implementation returns empty excludes
    }

    /// Called when a file matching the include patterns (and not excluded) is found.
    ///
    /// **Purpose**: Analyze a file to determine if it represents a project and extract its metadata.
    /// This is where the core project detection logic lives.
    ///
    /// **Performance Guidelines**:
    /// - Use filename/path checks before reading file contents
    /// - Return `None` quickly for files you can't process
    /// - Handle parse errors gracefully (return `None`, don't panic)
    /// - Minimize file I/O operations
    ///
    /// # Arguments
    ///
    /// * `workspace` - The current workspace context containing:
    ///   - `workspace.root`: Absolute path to workspace root
    ///   - `workspace.projects`: Previously discovered explicit projects (with marty.yml)
    ///   - `workspace.inferred_projects`: Previously discovered inferred projects
    /// * `path` - Absolute path to the file that matched your include patterns
    ///
    /// # Returns
    ///
    /// - `Some(InferredProject)` - If this file indicates a valid project
    /// - `None` - If this file should be ignored or couldn't be parsed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use marty_plugin_protocol::{WorkspaceProvider, Workspace, InferredProject};
    /// # use std::path::Path;
    /// # struct MyProvider;
    /// # impl WorkspaceProvider for MyProvider {
    /// # fn include_path_globs(&self) -> Vec<String> { vec![] }
    /// fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
    ///     // 1. Fast filename check
    ///     let filename = path.file_name()?.to_str()?;
    ///     if filename != "my-config.json" {
    ///         return None;
    ///     }
    ///     
    ///     // 2. Extract project information
    ///     let project_dir = path.parent()?.to_path_buf();
    ///     let name = project_dir.file_name()?.to_str()?.to_string();
    ///     
    ///     // 3. Parse file contents (handle errors gracefully)
    ///     let contents = std::fs::read_to_string(path).ok()?;
    ///     let config: serde_json::Value = serde_json::from_str(&contents).ok()?;
    ///     
    ///     // 4. Extract workspace dependencies (CRITICAL: only internal projects!)
    ///     let workspace_dependencies = vec![]; // Your logic here
    ///     
    ///     // 5. Return the discovered project
    ///     Some(InferredProject {
    ///         name,
    ///         project_dir,
    ///         discovered_by: "my-plugin".to_string(),
    ///         workspace_dependencies,
    ///     })
    /// }
    /// # }
    /// ```
    ///
    /// **Workspace Dependencies - Critical Guidelines**:
    ///
    /// The `workspace_dependencies` field must ONLY contain names of other projects
    /// within the same workspace, never external packages. This is used for:
    /// - Determining build order (dependencies built first)
    /// - Impact analysis (which projects are affected by changes)
    /// - Generating dependency graphs
    fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject>;
}

/// The main plugin trait that defines plugin metadata and capabilities.
///
/// **Purpose**: This trait provides Marty with information about your plugin and
/// connects it to the workspace provider implementation.
///
/// **Implementation Pattern**: Create a struct for your plugin and implement this trait.
///
/// # Example
///
/// ```rust
/// # use marty_plugin_protocol::{MartyPlugin, WorkspaceProvider, PluginType};
/// # use serde_json::{json, Value};
/// # struct MyWorkspaceProvider;
/// # impl WorkspaceProvider for MyWorkspaceProvider {
/// #   fn include_path_globs(&self) -> Vec<String> { vec![] }
/// #   fn on_file_found(&self, _: &marty_plugin_protocol::Workspace, _: &std::path::Path) -> Option<marty_plugin_protocol::InferredProject> { None }
/// # }
/// pub struct MyPlugin;
///
/// impl MartyPlugin for MyPlugin {
///     fn plugin_type(&self) -> PluginType {
///         PluginType::Primary
///     }
///
///     fn name(&self) -> &str {
///         "My Framework Plugin"
///     }
///
///     fn key(&self) -> &str {
///         "my-framework"
///     }
///
///     fn workspace_provider(&self) -> &dyn WorkspaceProvider {
///         &MyWorkspaceProvider
///     }
///
///     fn configuration_options(&self) -> Option<Value> {
///         Some(json!({
///             "type": "object",
///             "properties": {}
///         }))
///     }
/// }
/// ```
pub trait MartyPlugin {
    /// Return the type of this plugin.
    ///
    /// **Purpose**: Explicitly declares the plugin's role and capabilities.
    ///
    /// # Plugin Types
    ///
    /// - `PluginType::Primary`: Discovers projects (e.g., PNPM, Cargo, NPM)
    /// - `PluginType::Supplemental`: Enhances projects (e.g., TypeScript)
    /// - `PluginType::Hook`: Executes lifecycle actions (future feature)
    ///
    /// **Important**: The plugin type MUST match actual behavior:
    /// - `Primary` plugins MUST implement `include_path_globs()` and `on_file_found()`
    /// - `Supplemental` and `Hook` plugins MUST NOT discover projects
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::{MartyPlugin, PluginType};
    /// # struct PnpmPlugin;
    /// # impl MartyPlugin for PnpmPlugin {
    /// # fn name(&self) -> &str { "" }
    /// # fn key(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// fn plugin_type(&self) -> PluginType {
    ///     PluginType::Primary  // Discovers projects from package.json
    /// }
    /// # }
    /// ```
    fn plugin_type(&self) -> PluginType;

    /// Return the human-readable name of this plugin.
    ///
    /// **Purpose**: Displayed to users in logs, error messages, and plugin listings.
    /// Should be descriptive and professional.
    ///
    /// **Guidelines**:
    /// - Use proper capitalization and spacing
    /// - Include the technology/framework name
    /// - Keep it concise but descriptive
    /// - Avoid technical jargon that users might not understand
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::MartyPlugin;
    /// # struct MyPlugin;
    /// # impl MartyPlugin for MyPlugin {
    /// # fn plugin_type(&self) -> marty_plugin_protocol::PluginType { marty_plugin_protocol::PluginType::Primary }
    /// fn name(&self) -> &str {
    ///     "Cargo Workspace Plugin"      // ✅ Clear, professional
    ///     
    ///     // Other good examples:
    ///     // "TypeScript Project Plugin"   - Technology-specific
    ///     // "Python Requirements Plugin"  - Describes functionality
    ///     
    ///     // ❌ Avoid these patterns:
    ///     // "cargo-plugin"               - Too technical
    ///     // "My Super Awesome Plugin!!!" - Unprofessional
    ///     // "Plugin"                     - Too generic
    /// }
    /// # fn key(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// # }
    /// ```
    fn name(&self) -> &str;

    /// Return the unique identifier for this plugin.
    ///
    /// **Purpose**: Used internally by Marty for:
    /// - Plugin registration and loading
    /// - Configuration file sections (`[plugins.your-key]`)
    /// - Dependency resolution and caching
    /// - The `discovered_by` field in `InferredProject`
    ///
    /// **Requirements**:
    /// - Must be unique across all plugins
    /// - No whitespace characters (spaces, tabs, newlines)
    /// - Use kebab-case for consistency
    /// - Should be stable across plugin versions
    ///
    /// **Naming Convention**:
    /// - Use the primary technology name: `"cargo"`, `"typescript"`, `"python"`
    /// - For sub-technologies, use hyphens: `"next-js"`, `"create-react-app"`
    /// - Avoid version numbers or vendor names in the key
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::MartyPlugin;
    /// # struct MyPlugin;
    /// # impl MartyPlugin for MyPlugin {
    /// # fn plugin_type(&self) -> marty_plugin_protocol::PluginType { marty_plugin_protocol::PluginType::Primary }
    /// # fn name(&self) -> &str { "" }
    /// fn key(&self) -> &str {
    ///     "cargo"           // ✅ Rust Cargo projects
    ///     
    ///     // Other good examples:
    ///     // "typescript"      - TypeScript projects  
    ///     // "python"          - Python projects
    ///     // "next-js"         - Next.js frameworks
    ///     // "docker-compose"  - Docker Compose files
    ///     
    ///     // ❌ Avoid these patterns:
    ///     // "cargo plugin"    - Contains space
    ///     // "typescript-v2"   - Version in key
    ///     // "my company-ts"   - Vendor name + space
    /// }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// # }
    /// ```
    fn key(&self) -> &str;

    /// Return the workspace provider implementation for this plugin.
    ///
    /// **Purpose**: Connects your plugin to the workspace scanning and project
    /// detection logic. This is where the actual work happens.
    ///
    /// **Implementation Pattern**: Most plugins create a separate struct for the
    /// workspace provider and return a reference to a static instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use marty_plugin_protocol::{MartyPlugin, WorkspaceProvider};
    /// # struct MyPlugin;
    /// # struct MyWorkspaceProvider;
    /// # impl WorkspaceProvider for MyWorkspaceProvider {
    /// #   fn include_path_globs(&self) -> Vec<String> { vec![] }
    /// #   fn on_file_found(&self, _: &marty_plugin_protocol::Workspace, _: &std::path::Path) -> Option<marty_plugin_protocol::InferredProject> { None }
    /// # }
    /// # impl MartyPlugin for MyPlugin {
    /// # fn plugin_type(&self) -> marty_plugin_protocol::PluginType { marty_plugin_protocol::PluginType::Primary }
    /// # fn name(&self) -> &str { "" }
    /// # fn key(&self) -> &str { "" }
    /// fn workspace_provider(&self) -> &dyn WorkspaceProvider {
    ///     // Pattern 1: Static instance (most common)
    ///     &MyWorkspaceProvider
    ///     
    ///     // Pattern 2: If you need state, use a field
    ///     // &self.workspace_provider
    /// }
    /// # }
    /// ```
    fn workspace_provider(&self) -> &dyn WorkspaceProvider;

    /// Return JSON Schema configuration options for this plugin.
    ///
    /// **Purpose**: Allow users to customize plugin behavior through configuration files.
    /// The schema is used for validation and IDE autocomplete in Marty config files.
    ///
    /// **When to Provide Configuration**:
    /// - Plugin has configurable behavior (build commands, versions, paths)
    /// - Users need to specify framework-specific options
    /// - Different projects might need different settings
    ///
    /// **Schema Requirements**:
    /// - Must be valid JSON Schema (draft 7 recommended)
    /// - Should have a root `"type": "object"`
    /// - Include `"description"` for all properties
    /// - Provide sensible `"default"` values
    /// - Use `"additionalProperties": false` for strict validation
    ///
    /// # Example
    ///
    /// ```rust
    /// # use marty_plugin_protocol::MartyPlugin;
    /// # use serde_json::{json, Value};
    /// # struct MyPlugin;
    /// # impl MartyPlugin for MyPlugin {
    /// # fn plugin_type(&self) -> marty_plugin_protocol::PluginType { marty_plugin_protocol::PluginType::Primary }
    /// # fn name(&self) -> &str { "" }
    /// # fn key(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// fn configuration_options(&self) -> Option<Value> {
    ///     Some(json!({
    ///         "type": "object",
    ///         "description": "Configuration for My Plugin",
    ///         "properties": {
    ///             "enabled": {
    ///                 "type": "boolean",
    ///                 "description": "Enable this plugin",
    ///                 "default": true
    ///             },
    ///             "build_command": {
    ///                 "type": "string",
    ///                 "description": "Command to build projects",
    ///                 "default": "build"
    ///             }
    ///         },
    ///         "additionalProperties": false
    ///     }))
    /// }
    /// # }
    /// ```
    fn configuration_options(&self) -> Option<JsonValue> {
        None
    }
}
