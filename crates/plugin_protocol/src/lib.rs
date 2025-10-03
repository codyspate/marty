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
//! ## Plugin Types
//!
//! Marty supports three types of plugins, each with a distinct role:
//!
//! - **Primary**: Discovers projects and their workspace dependencies (e.g., PNPM, Cargo, NPM)
//! - **Supplemental**: Enhances existing projects without discovering new ones (e.g., TypeScript)
//! - **Hook**: Executes actions at lifecycle points without discovering projects (future: pre-commit hooks)
//!
//! See the [`PluginType`] enum for detailed descriptions and decision trees.
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
//! marty_plugin_protocol = "0.3"
//! serde_json = "1.0"
//! ```
//!
//! ### 2. Implement Your Plugin
//!
//! ```rust,no_run
//! use marty_plugin_protocol::{
//!     dylib::export_plugin, InferredProject, MartyPlugin, PluginType,
//!     Workspace, WorkspaceProvider,
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
//!         let workspace_dependencies = vec![]; // Your logic here
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
//! impl MartyPlugin for MyFrameworkPlugin {
//!     fn plugin_type(&self) -> PluginType {
//!         PluginType::Primary  // This plugin discovers projects
//!     }
//!
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
//! ## Module Organization
//!
//! - [`types`] - Core data structures (`PluginType`, `Project`, `InferredProject`, `Workspace`)
//! - [`traits`] - Plugin traits (`MartyPlugin`, `WorkspaceProvider`)
//! - [`message`] - Serializable message types for FFI communication
//! - [`dylib`] - Dynamic library export macro and C ABI interface
//!
//! ## Key Concepts
//!
//! ### Workspace Dependencies
//!
//! **Critical**: `workspace_dependencies` should ONLY contain names of projects within
//! the same workspace, NOT external packages from npm, crates.io, PyPI, etc.
//!
//! This is used for:
//! - **Task Ordering**: Determining the correct order to build/test projects
//! - **Dependency Resolution**: Understanding which projects depend on each other
//! - **Change Impact**: Knowing which projects are affected by changes
//!
//! ### Plugin Discovery Process
//!
//! 1. **Scanning**: Marty walks the workspace using your `include_path_globs()` patterns
//! 2. **Filtering**: Files matching `exclude_path_globs()` are skipped  
//! 3. **Detection**: `on_file_found()` is called for each matching file
//! 4. **Project Creation**: If a project is detected, an `InferredProject` is returned
//!
//! ## See Also
//!
//! - [Plugin Developer Guide](https://github.com/codyspate/marty/docs/PLUGIN_DEVELOPER_GUIDE.md)
//! - [Plugin Types Documentation](https://github.com/codyspate/marty/docs/PLUGIN_TYPES.md)
//! - [Plugin Examples](https://github.com/codyspate/marty/docs/PLUGIN_TYPE_EXAMPLES.md)

// Module declarations
mod message;
mod traits;
mod types;

// Re-export everything at the crate root for backward compatibility
pub use message::InferredProjectMessage;
pub use traits::{MartyPlugin, WorkspaceProvider};
pub use types::{InferredProject, PluginKey, PluginType, Project, Workspace};

// Dynamic library exports
pub mod dylib;
