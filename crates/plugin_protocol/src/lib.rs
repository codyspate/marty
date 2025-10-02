//! # Marty Plugin Protocol
//!
//! This crate provides everything needed to develop plugins for the Marty monorepo management tool.
//! Plugins are compiled as dynamic libraries (`.so`, `.dylib`, `.dll`) and implement the
//! `MartyPlugin` and `WorkspaceProvider` traits to add support for different project types and languages.
//!
//! ## Overview
//!
//! Marty plugins enable automatic discovery and management of projects within a monorepo.
//! Each plugin can:
//! - **Detect project files** using glob patterns
//! - **Parse project metadata** to extract names and dependencies
//! - **Identify workspace dependencies** between projects for proper task ordering
//! - **Provide configuration options** for customizing plugin behavior
//!
//! ## Quick Start Guide
//!
//! ### 1. Create a New Plugin Crate
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! name = "marty-plugin-myframework"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [lib]
//! crate-type = ["cdylib"]  # Required for dynamic library
//!
//! [dependencies]
//! marty_plugin_protocol = "0.2"
//! serde_json = "1.0"
//! ```
//!
//! ### 2. Implement Your Plugin
//!
//! ```rust,no_run
//! use marty_plugin_protocol::{
//!     dylib::export_plugin, InferredProject, MartyPlugin, Workspace, WorkspaceProvider,
//! };
//! use serde_json::{json, Value as JsonValue};
//! use std::path::Path;
//!
//! /// Main plugin struct
//! pub struct MyFrameworkPlugin;
//!
//! impl MyFrameworkPlugin {
//!     pub const fn new() -> Self {
//!         Self
//!     }
//! }
//!
//! impl Default for MyFrameworkPlugin {
//!     fn default() -> Self {
//!         Self::new()
//!     }
//! }
//!
//! /// Workspace provider for detecting projects
//! pub struct MyFrameworkWorkspaceProvider;
//!
//! impl WorkspaceProvider for MyFrameworkWorkspaceProvider {
//!     fn include_path_globs(&self) -> Vec<String> {
//!         vec!["**/my-framework.json".to_string()]
//!     }
//!
//!     fn exclude_path_globs(&self) -> Vec<String> {
//!         vec![
//!             "**/node_modules/**".to_string(),
//!             "**/target/**".to_string(),
//!             "**/.git/**".to_string(),
//!         ]
//!     }
//!
//!     fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
//!         // Only process our config files
//!         if path.file_name()?.to_str()? != "my-framework.json" {
//!             return None;
//!         }
//!
//!         let project_dir = path.parent()?.to_path_buf();
//!         let name = project_dir.file_name()?.to_str()?.to_string();
//!         
//!         // Parse the config file to extract workspace dependencies
//!         let workspace_dependencies = self.parse_dependencies(path, workspace);
//!
//!         Some(InferredProject {
//!             name,
//!             project_dir,
//!             discovered_by: "my-framework".to_string(),
//!             workspace_dependencies,
//!         })
//!     }
//! }
//!
//! impl MyFrameworkWorkspaceProvider {
//!     fn parse_dependencies(&self, _path: &Path, _workspace: &Workspace) -> Vec<String> {
//!         // Implementation details for parsing local workspace dependencies
//!         Vec::new()
//!     }
//! }
//!
//! impl MartyPlugin for MyFrameworkPlugin {
//!     fn name(&self) -> &str {
//!         "My Framework Plugin"
//!     }
//!
//!     fn key(&self) -> &str {
//!         "my-framework"
//!     }
//!
//!     fn workspace_provider(&self) -> &dyn WorkspaceProvider {
//!         &MyFrameworkWorkspaceProvider
//!     }
//!
//!     fn configuration_options(&self) -> Option<JsonValue> {
//!         Some(json!({
//!             "type": "object",
//!             "properties": {
//!                 "framework_version": {
//!                     "type": "string",
//!                     "description": "Framework version to target",
//!                     "default": "latest"
//!                 }
//!             },
//!             "additionalProperties": false
//!         }))
//!     }
//! }
//!
//! // Export the plugin - this creates the C ABI interface
//! export_plugin!(MyFrameworkPlugin);
//! ```
//!
//! ### 3. Build Your Plugin
//!
//! ```bash
//! cargo build --lib --release
//! ```
//!
//! The resulting dynamic library will be in `target/release/` with the appropriate
//! extension for your platform (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows).
//!
//! ## Core Concepts
//!
//! ### Plugin Discovery Process
//!
//! 1. **Scanning**: Marty walks the workspace using your `include_path_globs()` patterns
//! 2. **Filtering**: Files matching `exclude_path_globs()` are skipped  
//! 3. **Detection**: `on_file_found()` is called for each matching file
//! 4. **Project Creation**: If a project is detected, an `InferredProject` is returned
//!
//! ### Workspace Dependencies (Critical Concept)
//!
//! **⚠️ Important**: `workspace_dependencies` represents dependencies between projects
//! *within the same workspace*, NOT external packages. This is used for:
//!
//! - **Task Ordering**: Determining the correct order to build/test projects
//! - **Dependency Resolution**: Understanding which projects depend on each other
//! - **Change Impact**: Knowing which projects are affected by changes
//!
//! #### Examples of Workspace Dependencies:
//!
//! ```rust,no_run
//! # fn example_workspace_deps() -> Vec<String> {
//! // ✅ CORRECT: References to other projects in the workspace
//! vec![
//!     "shared-utils".to_string(),     // Another project in the workspace
//!     "common-types".to_string(),     // Shared library project
//!     "data-models".to_string(),      // Internal dependency
//! ]
//! # }
//!
//! # fn example_external_deps() -> Vec<String> {
//! // ❌ INCORRECT: External packages (don't include these)
//! vec![
//!     "serde".to_string(),           // External crate from crates.io
//!     "tokio".to_string(),           // External crate
//!     "@types/node".to_string(),     // External npm package
//!     "lodash".to_string(),          // External npm package
//! ]
//! # }
//! ```
//!
//! #### How to Detect Workspace Dependencies:
//!
//! Different project types have different ways to reference workspace projects:
//!
//! **Rust (Cargo.toml)**:
//! ```toml
//! [dependencies]
//! shared-utils = { path = "../shared-utils" }    # ✅ Workspace dependency
//! serde = "1.0"                                  # ❌ External dependency (ignore)
//! ```
//!
//! **JavaScript/TypeScript (package.json)**:
//! ```json
//! {
//!   "dependencies": {
//!     "@myworkspace/shared": "workspace:*",       // ✅ Workspace dependency
//!     "@myworkspace/utils": "file:../utils",     // ✅ Workspace dependency  
//!     "lodash": "^4.17.21"                       // ❌ External dependency (ignore)
//!   }
//! }
//! ```
//!
//! **Python (pyproject.toml)**:
//! ```toml
//! [build-system]
//! dependencies = [
//!     "shared-lib @ file:../shared-lib",         # ✅ Workspace dependency
//!     "requests>=2.25.0",                        # ❌ External dependency (ignore)
//! ]
//! ```
//!
//! ### Configuration Schema
//!
//! Plugins can define JSON Schema configuration options that users can set
//! in their Marty configuration. The schema should be complete and descriptive:
//!
//! ```rust,no_run
//! # use serde_json::{json, Value};
//! # fn configuration_example() -> Option<Value> {
//! Some(json!({
//!     "type": "object",
//!     "properties": {
//!         "build_command": {
//!             "type": "string",
//!             "description": "Command to build projects",
//!             "default": "build",
//!             "examples": ["build", "compile", "make"]
//!         },
//!         "target_directory": {
//!             "type": "string",
//!             "description": "Directory for build outputs relative to project root",
//!             "default": "dist"
//!         },
//!         "enable_optimization": {
//!             "type": "boolean",
//!             "description": "Enable build optimizations in release mode",
//!             "default": true
//!         },
//!         "supported_versions": {
//!             "type": "array",
//!             "description": "List of supported framework versions",
//!             "items": { "type": "string" },
//!             "default": ["1.0", "2.0"]
//!         },
//!         "exclude_patterns": {
//!             "type": "array",
//!             "description": "Additional glob patterns to exclude during scanning",
//!             "items": { "type": "string" },
//!             "default": []
//!         }
//!     },
//!     "additionalProperties": false,
//!     "required": []  // Specify required properties if any
//! }))
//! # }
//! ```
//!
//! ## Advanced Topics
//!
//! ### Error Handling
//!
//! Plugins should be resilient to malformed or missing files. The `on_file_found` method
//! should return `None` for files it can't process, allowing other plugins to handle them.
//!
//! ### Performance Considerations
//!
//! - **Minimize I/O**: Only read files you need to process
//! - **Fast Rejection**: Use filename checks before parsing file contents
//! - **Efficient Parsing**: Use streaming parsers for large files
//! - **Cache Results**: Consider caching parsed data if files are accessed multiple times
//!
//! ### Testing Your Plugin
//!
//! **Unit Testing**: Test your workspace provider logic in isolation:
//!
//! ```rust,no_run
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use std::path::PathBuf;
//!     use std::fs;
//!     use tempfile::TempDir;
//!
//!     #[test]
//!     fn test_project_detection() {
//!         let provider = MyFrameworkWorkspaceProvider;
//!         let workspace = Workspace {
//!             root: PathBuf::from("/workspace"),
//!             projects: vec![],
//!             inferred_projects: vec![],
//!         };
//!         
//!         let path = PathBuf::from("/workspace/project/my-framework.json");
//!         let result = provider.on_file_found(&workspace, &path);
//!         
//!         assert!(result.is_some());
//!         let project = result.unwrap();
//!         assert_eq!(project.name, "project");
//!         assert_eq!(project.discovered_by, "my-framework");
//!         assert!(project.workspace_dependencies.is_empty());
//!     }
//!
//!     #[test]
//!     fn test_workspace_dependencies() {
//!         // Create a temporary workspace with test files
//!         let temp_dir = TempDir::new().unwrap();
//!         let workspace_root = temp_dir.path();
//!         
//!         // Create a config file with dependencies
//!         let project_dir = workspace_root.join("my-project");
//!         fs::create_dir_all(&project_dir).unwrap();
//!         
//!         let config_file = project_dir.join("my-framework.json");
//!         fs::write(&config_file, r#"
//!         {
//!             "name": "my-project",
//!             "dependencies": {
//!                 "shared-utils": "workspace:*",
//!                 "lodash": "^4.0.0"
//!             }
//!         }"#).unwrap();
//!
//!         let workspace = Workspace {
//!             root: workspace_root.to_path_buf(),
//!             projects: vec![],
//!             inferred_projects: vec![
//!                 InferredProject {
//!                     name: "shared-utils".to_string(),
//!                     project_dir: workspace_root.join("shared-utils"),
//!                     discovered_by: "my-framework".to_string(),
//!                     workspace_dependencies: vec![],
//!                 }
//!             ],
//!         };
//!
//!         let provider = MyFrameworkWorkspaceProvider;
//!         let result = provider.on_file_found(&workspace, &config_file).unwrap();
//!         
//!         // Should detect workspace dependency but not external dependency
//!         assert_eq!(result.workspace_dependencies, vec!["shared-utils"]);
//!     }
//!
//!     #[test]
//!     fn test_include_patterns() {
//!         let provider = MyFrameworkWorkspaceProvider;
//!         let patterns = provider.include_path_globs();
//!         
//!         assert!(!patterns.is_empty());
//!         assert!(patterns.contains(&"**/my-framework.json".to_string()));
//!     }
//!
//!     #[test]
//!     fn test_exclude_patterns() {
//!         let provider = MyFrameworkWorkspaceProvider;
//!         let patterns = provider.exclude_path_globs();
//!         
//!         // Should exclude common build/cache directories
//!         assert!(patterns.iter().any(|p| p.contains("node_modules")));
//!     }
//! }
//! ```
//!
//! **Integration Testing**: Test with real project files:
//!
//! ```rust,no_run
//! #[cfg(test)]
//! mod integration_tests {
//!     use super::*;
//!     use std::process::Command;
//!
//!     #[test]
//!     fn test_real_project_files() {
//!         // Test with actual config files from your examples
//!         let test_data_dir = std::path::Path::new("tests/fixtures");
//!         
//!         for entry in std::fs::read_dir(test_data_dir).unwrap() {
//!             let path = entry.unwrap().path();
//!             if path.extension().and_then(|s| s.to_str()) == Some("json") {
//!                 let provider = MyFrameworkWorkspaceProvider;
//!                 let workspace = Workspace {
//!                     root: test_data_dir.to_path_buf(),
//!                     projects: vec![],
//!                     inferred_projects: vec![],
//!                 };
//!                 
//!                 // Should not panic on any real config file
//!                 let _ = provider.on_file_found(&workspace, &path);
//!             }
//!         }
//!     }
//!
//!     #[test]
//!     fn test_plugin_loading() {
//!         // Build the plugin and test that it can be loaded
//!         let output = Command::new("cargo")
//!             .args(&["build", "--lib", "--release"])
//!             .output()
//!             .expect("Failed to build plugin");
//!         
//!         assert!(output.status.success(), "Plugin failed to build");
//!         
//!         // Verify the dynamic library was created
//!         let lib_path = std::path::Path::new("target/release")
//!             .join(if cfg!(target_os = "linux") { "libmarty_plugin_my_framework.so" }
//!                   else if cfg!(target_os = "macos") { "libmarty_plugin_my_framework.dylib" }
//!                   else { "marty_plugin_my_framework.dll" });
//!         
//!         assert!(lib_path.exists(), "Dynamic library not found");
//!     }
//! }
//! ```
//!
//! ## Migration from WASM Plugins
//!
//! If you're migrating from the old WASM-based plugin system:
//!
//! ### Key Changes
//!
//! **Build Target**:
//! ```toml
//! # OLD: WASM target
//! # cargo build --target wasm32-wasip1
//!
//! # NEW: Native dynamic library
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! **Export Method**:
//! ```ignore
//! // OLD: Manual WASM exports
//! #[no_mangle]
//! pub extern "C" fn _start() { /* ... */ }
//!
//! // NEW: Simple macro
//! use marty_plugin_protocol::dylib::export_plugin;
//! struct MyPlugin;
//! export_plugin!(MyPlugin);
//! ```
//!
//! **Performance**:
//! - ✅ **Faster**: Native code execution vs WASM interpretation
//! - ✅ **Simpler**: No WASM runtime setup or sandboxing
//! - ✅ **Better debugging**: Native debugging tools work
//! - ✅ **Easier deployment**: Standard shared libraries
//!
//! **Dependencies**:
//! ```toml
//! # OLD: Limited to WASM-compatible crates
//! [dependencies]
//! serde = { version = "1.0", default-features = false }
//!
//! # NEW: Full crate ecosystem available
//! [dependencies]
//! serde_json = "1.0"
//! regex = "1.0"
//! # Any crate works!
//! ```
//!
//! ### Migration Steps
//!
//! 1. **Update Cargo.toml**:
//!    - Change `crate-type` to `["cdylib"]`
//!    - Remove WASM-specific profile settings
//!    - Add any previously unavailable dependencies
//!
//! 2. **Update plugin code**:
//!    - Remove manual WASM export functions
//!    - Add `export_plugin!(YourPlugin)` at the end of lib.rs
//!    - Ensure your plugin has a `new()` constructor
//!
//! 3. **Update build process**:
//!    - Remove WASM build commands
//!    - Use `cargo build --lib --release` instead
//!    - Look for `.so`/`.dylib`/`.dll` files in `target/release/`
//!
//! 4. **Test the migration**:
//!    - Copy the dynamic library to Marty's plugin directory
//!    - Verify plugin loading and project detection
//!    - Check that performance has improved
//!
//! ## Troubleshooting Guide
//!
//! ### Common Build Issues
//!
//! **"crate-type must be cdylib"**
//! ```toml
//! # Add this to your Cargo.toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! **"export_plugin macro not found"**
//! ```rust
//! // Make sure you import the macro
//! use marty_plugin_protocol::dylib::export_plugin;
//! ```
//!
//! **"Plugin struct must have new() method"**
//! ```rust
//! struct MyPlugin;
//!
//! impl MyPlugin {
//!     pub const fn new() -> Self {
//!         Self
//!     }
//! }
//! ```
//!
//! ### Runtime Issues
//!
//! **Plugin not detected by Marty:**
//! - Check that the plugin file is in the correct location (`~/.marty/plugins/`)
//! - Verify the file has the correct extension (`.so`, `.dylib`, or `.dll`)
//! - Ensure the plugin exports the required C symbols
//! - Check Marty logs for loading errors
//!
//! **Projects not being discovered:**
//! - Verify your `include_path_globs()` patterns match your target files
//! - Check that your patterns don't conflict with `exclude_path_globs()`
//! - Test your `on_file_found()` logic with sample files
//! - Ensure you're returning `Some(InferredProject)` for valid projects
//!
//! **Workspace dependencies not working:**
//! - Confirm you're only including internal workspace projects, not external packages
//! - Check that dependency names match the target projects' names exactly
//! - Verify the target projects exist in the workspace
//! - Review your dependency parsing logic for correctness
//!
//! ### Performance Issues
//!
//! **Slow workspace scanning:**
//! - Make include patterns as specific as possible
//! - Add exclude patterns for large directories (`node_modules`, `target`, etc.)
//! - Avoid reading file contents unless necessary
//! - Use filename checks before parsing
//!
//! **Memory usage:**
//! - Don't cache large amounts of data in plugin structs
//! - Let Marty handle string cleanup via `plugin_cleanup_string()`
//! - Avoid creating unnecessary allocations in hot paths
//!
//! ### Development Tips
//!
//! **Testing strategies:**
//! - Create unit tests for your workspace provider logic
//! - Use integration tests with real project files
//! - Test edge cases (malformed files, missing fields, etc.)
//! - Verify plugin loading with `cargo build --lib --release`
//!
//! **Debugging techniques:**
//! - Use `eprintln!()` for debug output (visible in Marty logs)
//! - Test workspace providers independently before plugin export
//! - Use `cargo expand` to inspect macro-generated code
//! - Enable Marty debug logging to see plugin interactions
//!
//! ## Data Structures Reference
//!
//! - [`MartyPlugin`] - Main plugin interface with metadata and configuration
//! - [`WorkspaceProvider`] - Plugin workspace scanning and detection logic
//! - [`PluginKey`] - Non-whitespace identifier for plugins
//! - [`Workspace`] - Workspace context passed to plugins during discovery
//! - [`Project`] - Represents a project with an explicit marty.yml config file  
//! - [`InferredProject`] - Represents a project discovered by a plugin
//! - [`InferredProjectMessage`] - Serializable version for plugin communication
//! - [`export_plugin!`] - Macro to export your plugin with C ABI interface

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a project with an explicit `marty.yml` or `marty.yaml` configuration file.
///
/// These are "explicit" projects because they have been manually configured by the user
/// with a Marty configuration file. They take precedence over inferred projects and
/// can override plugin-detected settings.
///
/// **When Projects Are Created**:
/// - User creates a `marty.yml` or `marty.yaml` file in a directory
/// - The file contains explicit project configuration (name, dependencies, tasks, etc.)
/// - Marty automatically discovers these during workspace scanning
///
/// **Relationship to Plugins**: Plugins typically don't create `Project` instances directly.
/// Instead, they create `InferredProject` instances for projects they discover automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The name of the project as specified in the marty.yml file.
    ///
    /// This takes precedence over any name that might be inferred by plugins.
    pub name: String,

    /// Absolute path to the directory containing the project.
    ///
    /// This is typically the directory containing the marty.yml file.
    pub project_dir: PathBuf,

    /// Optional path to the marty.yml file that defines this project.
    ///
    /// Will be `None` if the project was created programmatically,
    /// or `Some(path)` if it was loaded from a configuration file.
    pub file_path: Option<PathBuf>,
}

/// Represents a project automatically discovered by a plugin without explicit Marty configuration.
///
/// This is the primary data structure that plugins return from `WorkspaceProvider::on_file_found()`.
/// Inferred projects represent real projects in the workspace that plugins can automatically
/// detect and understand based on their configuration files (package.json, Cargo.toml, etc.).
///
/// **Purpose**: Enable Marty to automatically understand the structure of a monorepo without
/// requiring manual configuration for every project.
///
/// **Lifecycle**:
/// 1. Plugin detects a project file (e.g., package.json)
/// 2. Plugin parses the file to extract metadata
/// 3. Plugin creates and returns an `InferredProject`
/// 4. Marty incorporates the project into the workspace dependency graph
/// 5. Marty uses the information for task execution, change detection, etc.
///
/// # Example Creation
///
/// ```rust
/// # use marty_plugin_protocol::InferredProject;
/// # use std::path::PathBuf;
/// let project = InferredProject {
///     name: "my-api-server".to_string(),
///     project_dir: PathBuf::from("/workspace/services/api"),
///     discovered_by: "cargo".to_string(),
///     workspace_dependencies: vec![
///         "shared-models".to_string(),    // Another project in workspace
///         "common-utils".to_string(),     // Another project in workspace
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredProject {
    /// The name of the project as determined by the plugin.
    ///
    /// **Guidelines for Naming**:
    /// - Use the project's natural name from its configuration (e.g., `name` field in package.json)
    /// - Fall back to the directory name if no explicit name is found
    /// - Ensure names are unique within the workspace (plugin responsibility)
    /// - Use kebab-case or the project's native naming convention
    /// - Avoid special characters that might cause issues in file paths or commands
    ///
    /// **Examples**:
    /// - `"user-service"` - From package.json name field
    /// - `"data-models"` - From Cargo.toml package.name
    /// - `"frontend-app"` - Directory name if no config name available
    pub name: String,

    /// Absolute path to the directory containing the project.
    ///
    /// **Requirements**:
    /// - Must be an absolute path
    /// - Should point to the root directory of the project
    /// - Typically the directory containing the main project configuration file
    /// - Must be within the workspace root directory
    ///
    /// **Determining Project Directory**:
    /// - For `package.json`: Directory containing the package.json file
    /// - For `Cargo.toml`: Directory containing the Cargo.toml file  
    /// - For `pyproject.toml`: Directory containing the pyproject.toml file
    /// - For multi-file projects: The "root" directory that contains the main config
    ///
    /// **Example**:
    /// ```rust
    /// # use std::path::{Path, PathBuf};
    /// # fn example(config_file_path: &Path) -> PathBuf {
    /// // If config file is at /workspace/services/api/package.json
    /// // Then project_dir should be /workspace/services/api
    /// config_file_path.parent().unwrap().to_path_buf()
    /// # }
    /// ```
    pub project_dir: PathBuf,

    /// Identifier of the plugin that discovered this project.
    ///
    /// **Purpose**:
    /// - Traceability: Know which plugin detected each project
    /// - Debugging: Identify plugin-specific issues
    /// - Plugin management: Handle conflicts between plugins
    /// - User feedback: Show users what was detected automatically
    ///
    /// **Requirements**:
    /// - Must exactly match the plugin's `key()` return value
    /// - Should be the same across all projects detected by the plugin
    /// - Must not contain whitespace (inherited from plugin key requirements)
    ///
    /// **Example**:
    /// ```rust
    /// # struct MyPlugin;
    /// # impl marty_plugin_protocol::MartyPlugin for MyPlugin {
    /// fn key(&self) -> &str {
    ///     "cargo"  // This exact string should be used in discovered_by
    /// }
    /// # fn name(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// # }
    /// ```
    pub discovered_by: String,

    /// List of other projects in the same workspace that this project depends on.
    ///
    /// **⚠️ CRITICAL: Only Internal Dependencies**
    ///
    /// This field must ONLY contain names of other projects within the same workspace,
    /// never external packages or libraries. This is used for:
    ///
    /// - **Build Ordering**: Ensuring dependencies are built before dependents
    /// - **Change Impact**: Determining which projects are affected by changes
    /// - **Task Orchestration**: Running tasks in the correct dependency order
    /// - **Workspace Analysis**: Understanding the internal project structure
    ///
    /// **What to Include** ✅:
    /// - Other projects in the workspace that this project imports/uses
    /// - Path-based dependencies (`../shared-lib`, `file:../utils`)
    /// - Workspace protocol dependencies (`workspace:*`)
    /// - Monorepo-internal package references
    ///
    /// **What NOT to Include** ❌:
    /// - External packages from registries (npm, crates.io, PyPI, etc.)
    /// - System libraries or built-in modules
    /// - Third-party dependencies
    /// - Development tools or build dependencies
    ///
    /// **Detection Examples by Language**:
    ///
    /// ```toml
    /// # Cargo.toml - Extract workspace deps
    /// [dependencies]
    /// shared-models = { path = "../shared-models" }  # ✅ Include "shared-models"
    /// serde = "1.0"                                  # ❌ External - ignore
    /// ```
    ///
    /// ```json
    /// // package.json - Extract workspace deps  
    /// {
    ///   "dependencies": {
    ///     "@myorg/shared": "workspace:*",           // ✅ Include "shared"
    ///     "@myorg/utils": "file:../utils",          // ✅ Include "utils"
    ///     "lodash": "^4.17.21"                      // ❌ External - ignore
    ///   }
    /// }
    /// ```
    ///
    /// ```toml
    /// # pyproject.toml - Extract workspace deps
    /// [tool.poetry.dependencies]
    /// shared-lib = { path = "../shared-lib" }       # ✅ Include "shared-lib"
    /// requests = "^2.25.0"                          # ❌ External - ignore
    /// ```
    ///
    /// **Name Resolution**: The names in this list should match the `name` field
    /// of the target projects' `InferredProject` or `Project` structs.
    ///
    /// **Validation**: Marty will validate that all dependencies actually exist
    /// in the workspace and warn about missing dependencies.
    pub workspace_dependencies: Vec<String>,
}

/// A workspace containing all discovered projects and their metadata.
///
/// This structure represents the complete view of a monorepo workspace as understood by Marty.
/// It is passed to plugins during project discovery to provide context about the workspace
/// and other projects that have already been discovered.
///
/// **Plugin Usage**: The `Workspace` is passed as a parameter to `WorkspaceProvider::on_file_found()`
/// to give plugins access to:
/// - The workspace root directory for path resolution
/// - Previously discovered explicit projects (with marty.yml files)
/// - Previously discovered inferred projects (from other plugins or earlier scans)
///
/// **Immutability**: During plugin execution, the workspace is read-only. Plugins cannot
/// modify the workspace directly - they return new `InferredProject` instances instead.
///
/// # Example Usage in Plugin
///
/// ```rust,no_run
/// # use marty_plugin_protocol::{Workspace, InferredProject, WorkspaceProvider};
/// # use std::path::Path;
/// # struct MyProvider;
/// # impl WorkspaceProvider for MyProvider {
/// # fn include_path_globs(&self) -> Vec<String> { vec![] }
/// fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
///     // Check if we're in the workspace root
///     if !path.starts_with(&workspace.root) {
///         return None;
///     }
///     
///     // Get relative path for logging
///     let rel_path = path.strip_prefix(&workspace.root).ok()?;
///     println!("Processing: {}", rel_path.display());
///     
///     // Check if this project name already exists
///     let project_name = path.parent()?.file_name()?.to_str()?.to_string();
///     let name_exists = workspace.projects.iter().any(|p| p.name == project_name)
///         || workspace.inferred_projects.iter().any(|p| p.name == project_name);
///     
///     if name_exists {
///         println!("Project '{}' already exists, skipping", project_name);
///         return None;
///     }
///     
///     // Find workspace dependencies by checking other projects
///     let workspace_dependencies = self.find_dependencies(&project_name, workspace);
///     
///     Some(InferredProject {
///         name: project_name,
///         project_dir: path.parent()?.to_path_buf(),
///         discovered_by: "my-plugin".to_string(),
///         workspace_dependencies,
///     })
/// }
/// # }
/// # impl MyProvider {
/// #   fn find_dependencies(&self, _name: &str, _workspace: &Workspace) -> Vec<String> { vec![] }
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    /// Absolute path to the root directory of the workspace.
    ///
    /// **Purpose**: Serves as the base path for all relative path operations and ensures
    /// all projects are contained within the workspace boundary.
    ///
    /// **Properties**:
    /// - Always an absolute path
    /// - All projects (explicit and inferred) must be subdirectories of this root
    /// - Used for path resolution and relative path calculations
    /// - Typically the directory containing the top-level Marty configuration
    ///
    /// **Plugin Usage**:
    /// - Validate that discovered projects are within the workspace
    /// - Resolve relative paths in configuration files
    /// - Calculate project relationships based on directory structure
    pub root: PathBuf,

    /// List of explicit projects with marty.yml/marty.yaml configuration files.
    ///
    /// **Purpose**: Projects that have been explicitly configured by users with Marty
    /// configuration files. These take precedence over inferred projects.
    ///
    /// **Properties**:
    /// - Discovered by scanning for marty.yml/marty.yaml files
    /// - Have explicit configuration (name, dependencies, tasks, etc.)
    /// - Take precedence over inferred projects with the same name
    /// - May override or supplement plugin-detected information
    ///
    /// **Plugin Considerations**:
    /// - Check if an explicit project already exists before creating an inferred one
    /// - Explicit projects may have different names than what plugins would infer
    /// - Plugins should respect explicit project boundaries and configurations
    pub projects: Vec<Project>,

    /// List of projects automatically discovered by plugins.
    ///
    /// **Purpose**: Projects that have been automatically detected by plugins based on
    /// configuration files (package.json, Cargo.toml, etc.) without explicit Marty configuration.
    ///
    /// **Properties**:
    /// - Discovered by plugin scanning logic
    /// - May be from different plugins (multiple plugins can discover different aspects)
    /// - Built up incrementally as plugins process files
    /// - Used to avoid duplicate detection and name conflicts
    ///
    /// **Plugin Usage**:
    /// - Check for existing inferred projects to avoid duplicates
    /// - Reference other projects when determining workspace dependencies  
    /// - Validate that workspace dependencies refer to real projects
    ///
    /// **Note**: During `on_file_found()` execution, this list contains projects
    /// discovered earlier in the scan, but not projects that will be discovered later.
    pub inferred_projects: Vec<InferredProject>,
    // Note: dependency graph is intentionally omitted for plugin interface simplicity
    // Plugins work with the raw project lists and Marty builds the dependency graph separately
}

/// The main interface that all workspace provider plugins must implement.
///
/// This trait defines how a plugin discovers and analyzes projects within a workspace.
/// Implementations should be efficient and handle malformed files gracefully.
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
    /// # fn exclude_path_globs(&self) -> Vec<String> { vec![] }
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
    ///     let workspace_dependencies = self.extract_workspace_deps(&config, workspace);
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
    /// # impl MyProvider {
    /// #   fn extract_workspace_deps(&self, _config: &serde_json::Value, _workspace: &Workspace) -> Vec<String> { vec![] }
    /// # }
    /// ```
    ///
    /// **Workspace Dependencies - Critical Guidelines**:
    ///
    /// The `workspace_dependencies` field must ONLY contain names of other projects
    /// within the same workspace, never external packages. This is used for:
    /// - Determining build order (dependencies built first)
    /// - Impact analysis (which projects are affected by changes)
    /// - Task orchestration across the workspace
    ///
    /// To find workspace dependencies, look for:
    /// - **Path-based references**: `"../other-project"`, `"file:../utils"`
    /// - **Workspace protocols**: `"workspace:*"`, `"workspace:^1.0.0"`
    /// - **Local imports**: Projects that import from sibling directories
    /// - **Monorepo-specific syntax**: Framework-specific workspace dependency syntax
    fn on_file_found(
        &self,
        workspace: &Workspace,
        path: &std::path::Path,
    ) -> Option<InferredProject>;
}

/// A plugin key that ensures no whitespace characters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginKey(String);

impl PluginKey {
    /// Create a new plugin key, filtering out any whitespace characters
    pub fn new(key: impl Into<String>) -> Self {
        let key = key
            .into()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        Self(key)
    }

    /// Get the key as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The main plugin interface that defines plugin metadata and behavior.
///
/// This trait provides Marty with essential information about your plugin and
/// connects it to the workspace scanning logic via `WorkspaceProvider`.
///
/// # Implementation Requirements
///
/// Your plugin struct must also implement:
/// - `Default` or provide a `new()` constructor for the `export_plugin!` macro
/// - Be `Send + Sync` for thread safety (automatically satisfied for most structs)
///
/// # Example
///
/// ```rust
/// use marty_plugin_protocol::{MartyPlugin, WorkspaceProvider};
/// use serde_json::{json, Value};
///
/// pub struct MyPlugin;
///
/// impl MyPlugin {
///     pub const fn new() -> Self { Self }
/// }
///
/// impl Default for MyPlugin {
///     fn default() -> Self { Self::new() }  
/// }
///
/// # struct MyWorkspaceProvider;
/// # impl WorkspaceProvider for MyWorkspaceProvider {
/// #   fn include_path_globs(&self) -> Vec<String> { vec![] }
/// #   fn on_file_found(&self, _: &marty_plugin_protocol::Workspace, _: &std::path::Path) -> Option<marty_plugin_protocol::InferredProject> { None }
/// # }
/// impl MartyPlugin for MyPlugin {
///     fn name(&self) -> &str {
///         "My Awesome Framework Plugin"  // User-friendly display name
///     }
///     
///     fn key(&self) -> &str {
///         "my-framework"  // Unique identifier (no spaces!)
///     }
///     
///     fn workspace_provider(&self) -> &dyn WorkspaceProvider {
///         &MyWorkspaceProvider  // Your detection logic
///     }
///     
///     fn configuration_options(&self) -> Option<Value> {
///         Some(json!({
///             "type": "object",
///             "properties": {
///                 "version": {
///                     "type": "string",
///                     "description": "Target framework version",
///                     "default": "latest"
///                 }
///             },
///             "additionalProperties": false
///         }))
///     }
/// }
/// ```
pub trait MartyPlugin {
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
    /// # Example: Comprehensive Configuration Schema
    ///
    /// ```rust
    /// # use marty_plugin_protocol::MartyPlugin;
    /// # use serde_json::{json, Value};
    /// # struct MyPlugin;
    /// # impl MartyPlugin for MyPlugin {
    /// # fn name(&self) -> &str { "" }
    /// # fn key(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// fn configuration_options(&self) -> Option<Value> {
    ///     Some(json!({
    ///         "type": "object",
    ///         "description": "Configuration for My Framework Plugin",
    ///         "properties": {
    ///             // String configurations
    ///             "build_command": {
    ///                 "type": "string",
    ///                 "description": "Command to build projects",
    ///                 "default": "build",
    ///                 "examples": ["build", "compile", "make"]
    ///             },
    ///             "target_version": {
    ///                 "type": "string",
    ///                 "description": "Framework version to target",
    ///                 "default": "latest",
    ///                 "enum": ["1.0", "2.0", "latest"]
    ///             },
    ///             
    ///             // Boolean flags
    ///             "enable_optimization": {
    ///                 "type": "boolean",
    ///                 "description": "Enable build optimizations",
    ///                 "default": true
    ///             },
    ///             "strict_mode": {
    ///                 "type": "boolean",
    ///                 "description": "Enable strict validation rules",
    ///                 "default": false
    ///             },
    ///             
    ///             // Array configurations
    ///             "additional_includes": {
    ///                 "type": "array",
    ///                 "description": "Extra glob patterns to include",
    ///                 "items": { "type": "string" },
    ///                 "default": [],
    ///                 "examples": [["**/*.config.js", "**/*.env"]]
    ///             },
    ///             
    ///             // Nested object configurations
    ///             "build_settings": {
    ///                 "type": "object",
    ///                 "description": "Advanced build configuration",
    ///                 "properties": {
    ///                     "output_dir": {
    ///                         "type": "string",
    ///                         "description": "Build output directory",
    ///                         "default": "dist"
    ///                     },
    ///                     "minify": {
    ///                         "type": "boolean",
    ///                         "description": "Minify output files",
    ///                         "default": true
    ///                     }
    ///                 },
    ///                 "additionalProperties": false
    ///             }
    ///         },
    ///         "additionalProperties": false,
    ///         "required": []  // Specify required fields if any
    ///     }))
    /// }
    /// # }
    /// ```
    ///
    /// **For Simple Plugins**: Return `None` if your plugin doesn't need configuration.
    ///
    /// ```rust
    /// # use marty_plugin_protocol::MartyPlugin;
    /// # use serde_json::Value;
    /// # struct SimplePlugin;
    /// # impl MartyPlugin for SimplePlugin {
    /// # fn name(&self) -> &str { "" }
    /// # fn key(&self) -> &str { "" }
    /// # fn workspace_provider(&self) -> &dyn marty_plugin_protocol::WorkspaceProvider { todo!() }
    /// fn configuration_options(&self) -> Option<Value> {
    ///     None  // No configuration needed for this plugin
    /// }
    /// # }
    /// ```
    fn configuration_options(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Serializable message format for WASM plugin communication
#[derive(Debug, Serialize, Deserialize)]
pub struct InferredProjectMessage {
    pub name: String,
    pub project_dir: String,
    pub discovered_by: String,
    pub workspace_dependencies: Vec<String>,
}

impl InferredProjectMessage {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        project_dir: impl Into<String>,
        discovered_by: impl Into<String>,
        workspace_dependencies: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            project_dir: project_dir.into(),
            discovered_by: discovered_by.into(),
            workspace_dependencies,
        }
    }
}

impl From<InferredProject> for InferredProjectMessage {
    fn from(project: InferredProject) -> Self {
        Self {
            name: project.name,
            project_dir: project.project_dir.display().to_string(),
            discovered_by: project.discovered_by,
            workspace_dependencies: project.workspace_dependencies,
        }
    }
}

impl From<InferredProjectMessage> for InferredProject {
    fn from(message: InferredProjectMessage) -> Self {
        Self {
            name: message.name,
            project_dir: PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        }
    }
}

/// Dynamic library plugin interface
///
/// This module provides the essential infrastructure for creating C ABI compatible plugins
/// that can be loaded as dynamic libraries (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows).
///
/// **Why Dynamic Libraries?**: Dynamic libraries provide better performance, easier debugging,
/// and simpler deployment compared to WASM while maintaining cross-platform compatibility.
pub mod dylib {

    /// Macro to export your plugin with a C ABI interface for dynamic library loading.
    ///
    /// **Purpose**: This macro generates the necessary C-compatible functions that Marty
    /// needs to interact with your plugin. It creates a bridge between Rust's type system
    /// and the C ABI required for cross-language dynamic loading.
    ///
    /// **Requirements**: Your plugin type must:
    /// - Implement the `MartyPlugin` trait  
    /// - Have a `const fn new() -> Self` constructor
    /// - Be `Send + Sync` (automatically satisfied for most structs)
    ///
    /// **Generated Functions**: The macro creates these C ABI exports:
    /// - `plugin_name()` - Returns the plugin's display name
    /// - `plugin_key()` - Returns the plugin's unique identifier
    /// - `plugin_on_file_found()` - Handles file discovery events
    /// - `plugin_cleanup_string()` - Manages memory for returned strings
    /// - `plugin_configuration_options()` - Returns JSON schema configuration
    ///
    /// # Usage
    ///
    /// ```rust
    /// use marty_plugin_protocol::{
    ///     dylib::export_plugin, InferredProject, MartyPlugin, Workspace, WorkspaceProvider,
    /// };
    /// use serde_json::{json, Value as JsonValue};
    /// use std::path::Path;
    ///
    /// // 1. Define your plugin struct
    /// pub struct MyPlugin;
    ///
    /// impl MyPlugin {
    ///     pub const fn new() -> Self {
    ///         Self
    ///     }
    /// }
    ///
    /// impl Default for MyPlugin {
    ///     fn default() -> Self {
    ///         Self::new()
    ///     }
    /// }
    ///
    /// // 2. Define your workspace provider  
    /// pub struct MyWorkspaceProvider;
    ///
    /// impl WorkspaceProvider for MyWorkspaceProvider {
    ///     fn include_path_globs(&self) -> Vec<String> {
    ///         vec!["**/my-config.json".to_string()]
    ///     }
    ///
    ///     fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
    ///         // Your detection logic here
    ///         None
    ///     }
    /// }
    ///
    /// // 3. Implement the MartyPlugin trait
    /// impl MartyPlugin for MyPlugin {
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
    ///     fn configuration_options(&self) -> Option<JsonValue> {
    ///         Some(json!({
    ///             "type": "object",
    ///             "properties": {
    ///                 "version": {
    ///                     "type": "string",
    ///                     "description": "Framework version",
    ///                     "default": "latest"
    ///                 }
    ///             },
    ///             "additionalProperties": false
    ///         }))
    ///     }
    /// }
    ///
    /// // 4. Export the plugin - this MUST be the last line in your lib.rs
    /// export_plugin!(MyPlugin);
    /// ```
    ///
    /// # Memory Management
    ///
    /// The macro handles C string memory management automatically:
    /// - Strings returned to Marty are allocated with `CString::into_raw()`
    /// - Marty calls `plugin_cleanup_string()` to free the memory safely
    /// - Plugins should never manually free strings returned to Marty
    ///
    /// # Error Handling  
    ///
    /// The generated C functions handle errors gracefully:
    /// - Invalid UTF-8 strings return null pointers
    /// - Parse errors in `on_file_found()` return null (no project detected)
    /// - Memory allocation failures return null pointers
    /// - Marty treats null returns as "no result" rather than errors
    ///
    /// # Build Configuration
    ///
    /// Your `Cargo.toml` must specify `cdylib` as the crate type:
    ///
    /// ```toml
    /// [lib]
    /// crate-type = ["cdylib"]
    /// ```
    ///
    /// # Platform Compatibility
    ///
    /// The generated dynamic library works on:
    /// - **Linux**: `.so` files (e.g., `libmarty_plugin_cargo.so`)
    /// - **macOS**: `.dylib` files (e.g., `libmarty_plugin_cargo.dylib`)  
    /// - **Windows**: `.dll` files (e.g., `marty_plugin_cargo.dll`)
    ///
    /// Marty automatically handles the platform-specific loading and naming conventions.
    ///
    /// # Testing Your Plugin
    ///
    /// After building with `cargo build --lib --release`, you can test the plugin by:
    /// 1. Placing the dynamic library in Marty's plugin directory
    /// 2. Running Marty in a workspace with your target project type
    /// 3. Checking that your projects are detected correctly
    ///
    /// **Plugin Directory Locations**:
    /// - Linux/macOS: `~/.marty/plugins/`
    /// - Windows: `%APPDATA%\marty\plugins\`
    ///
    /// # Common Issues
    ///
    /// **"Plugin not found"**: Ensure your Cargo.toml has `crate-type = ["cdylib"]`
    ///
    /// **"Symbol not found"**: Verify `export_plugin!(YourPluginType)` is called exactly once
    ///
    /// **"Memory errors"**: Never manually free strings returned to Marty; the macro handles this
    ///
    /// **"Projects not detected"**: Check your `include_path_globs()` patterns match your target files
    #[macro_export]
    macro_rules! export_plugin {
        ($plugin_type:ty) => {
            use std::ffi::{CStr, CString};
            use std::os::raw::c_char;

            static PLUGIN: $plugin_type = <$plugin_type>::new();

            #[no_mangle]
            pub extern "C" fn plugin_name() -> *const c_char {
                let name = PLUGIN.name();
                match CString::new(name) {
                    Ok(cstr) => cstr.into_raw(),
                    Err(_) => std::ptr::null(),
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_key() -> *const c_char {
                let key = PLUGIN.key();
                match CString::new(key) {
                    Ok(cstr) => cstr.into_raw(),
                    Err(_) => std::ptr::null(),
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_include_globs() -> *const c_char {
                let globs = PLUGIN.workspace_provider().include_path_globs();
                match serde_json::to_string(&globs) {
                    Ok(json) => match CString::new(json) {
                        Ok(cstr) => cstr.into_raw(),
                        Err(_) => std::ptr::null(),
                    },
                    Err(_) => std::ptr::null(),
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_exclude_globs() -> *const c_char {
                let globs = PLUGIN.workspace_provider().exclude_path_globs();
                match serde_json::to_string(&globs) {
                    Ok(json) => match CString::new(json) {
                        Ok(cstr) => cstr.into_raw(),
                        Err(_) => std::ptr::null(),
                    },
                    Err(_) => std::ptr::null(),
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_config_options() -> *const c_char {
                match PLUGIN.configuration_options() {
                    Some(options) => match serde_json::to_string(&options) {
                        Ok(json) => match CString::new(json) {
                            Ok(cstr) => cstr.into_raw(),
                            Err(_) => std::ptr::null(),
                        },
                        Err(_) => std::ptr::null(),
                    },
                    None => std::ptr::null(),
                }
            }

            /// Safe wrapper for handling plugin file found logic
            fn handle_file_found_safe(
                path_ptr: *const c_char,
                _contents_ptr: *const c_char,
            ) -> Option<*const c_char> {
                // Validate pointers before any unsafe operations
                if path_ptr.is_null() {
                    return None;
                }

                // Extract path string safely
                let path_str = unsafe {
                    let path_cstr = CStr::from_ptr(path_ptr);
                    match path_cstr.to_str() {
                        Ok(s) => s,
                        Err(_) => return None,
                    }
                };

                let path = std::path::Path::new(path_str);

                // Create a minimal workspace context for the plugin
                let workspace = marty_plugin_protocol::Workspace {
                    root: std::path::PathBuf::from("."),
                    projects: Vec::new(),
                    inferred_projects: Vec::new(),
                };

                match PLUGIN.workspace_provider().on_file_found(&workspace, path) {
                    Some(project) => {
                        let message = marty_plugin_protocol::InferredProjectMessage::from(project);
                        match serde_json::to_string(&message) {
                            Ok(json) => match CString::new(json) {
                                Ok(cstr) => Some(cstr.into_raw()),
                                Err(_) => None,
                            },
                            Err(_) => None,
                        }
                    }
                    None => match CString::new("null") {
                        Ok(cstr) => Some(cstr.into_raw()),
                        Err(_) => None,
                    },
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_on_file_found(
                path_ptr: *const c_char,
                contents_ptr: *const c_char,
            ) -> *const c_char {
                handle_file_found_safe(path_ptr, contents_ptr).unwrap_or_else(std::ptr::null)
            }

            /// Safe wrapper for cleaning up plugin-allocated strings
            fn cleanup_string_safe(ptr: *const c_char) {
                if !ptr.is_null() {
                    unsafe {
                        let _ = CString::from_raw(ptr as *mut c_char);
                    }
                }
            }

            #[no_mangle]
            pub extern "C" fn plugin_cleanup_string(ptr: *const c_char) {
                cleanup_string_safe(ptr);
            }
        };
    }

    pub use export_plugin;
}
