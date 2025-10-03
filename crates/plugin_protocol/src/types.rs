//! Core types for the Marty plugin protocol.
//!
//! This module contains the fundamental data structures used throughout the plugin system:
//! - [`PluginType`] - Defines plugin capabilities (Primary, Supplemental, Hook)
//! - [`Project`] - Explicit projects with marty.yml configuration
//! - [`InferredProject`] - Projects discovered automatically by plugins
//! - [`Workspace`] - The workspace context containing all projects
//! - [`PluginKey`] - Type-safe plugin identifier

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Defines the type and capabilities of a Marty plugin.
///
/// ## Overview
///
/// Plugin types make plugin roles explicit, enabling type safety, optimization,
/// and clear documentation of plugin capabilities. Each type has distinct responsibilities
/// and behavioral expectations.
///
/// ## Plugin Types
///
/// ### Primary
/// **Discovers projects and their workspace dependencies**
///
/// Primary plugins scan the workspace for project files and create `InferredProject` instances.
/// They are the source of truth for project structure and dependencies.
///
/// **Use when your plugin**:
/// - Scans for configuration files (e.g., `Cargo.toml`, `package.json`)
/// - Creates new project entries in the workspace
/// - Detects workspace dependencies between projects
///
/// **Examples**: PNPM plugin, Cargo plugin, NPM plugin, Python plugin
///
/// ### Supplemental
/// **Enhances existing projects without discovering new ones**
///
/// Supplemental plugins add functionality to projects discovered by Primary plugins.
/// They never create projects themselves.
///
/// **Use when your plugin**:
/// - Generates or updates configuration files
/// - Adds framework-specific tooling
/// - Enhances projects discovered by other plugins
///
/// **Examples**: TypeScript plugin (adds project references to PNPM-discovered projects)
///
/// ### Hook
/// **Executes actions at lifecycle points** (future feature)
///
/// Hook plugins run commands or scripts at specific points in the workspace lifecycle.
///
/// **Use when your plugin**:
/// - Runs pre-commit or post-build checks
/// - Validates workspace state
/// - Integrates with external tools
///
/// **Examples** (planned): Pre-commit hooks, linters, formatters, deployment scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// Plugin discovers projects and their workspace dependencies.
    Primary,
    /// Plugin enhances existing projects without discovering new ones.
    Supplemental,
    /// Plugin executes actions at lifecycle hooks without discovering projects.
    Hook,
}

impl PluginType {
    /// Returns whether this plugin type is expected to discover projects.
    #[must_use]
    pub const fn discovers_projects(&self) -> bool {
        matches!(self, Self::Primary)
    }

    /// Returns a human-readable description of this plugin type.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Primary => "Discovers projects and workspace dependencies",
            Self::Supplemental => "Enhances existing projects without discovering new ones",
            Self::Hook => "Executes actions at lifecycle hooks",
        }
    }
}

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

    /// Project dependencies explicitly declared in marty.yml.
    ///
    /// These are names of other projects within the workspace that this project depends on.
    /// Dependencies control task execution order - dependency projects are processed before
    /// dependent projects.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Represents a project automatically discovered by a plugin without explicit Marty configuration.
///
/// **Purpose**: Plugins create `InferredProject` instances when they detect projects
/// in the workspace based on framework-specific files (package.json, Cargo.toml, etc.).
///
/// **Important Distinction**: Unlike `Project` which has an explicit marty.yml file,
/// inferred projects are detected automatically. However, if a directory has both
/// framework files AND a marty.yml, the explicit `Project` configuration takes precedence.
///
/// **Creating Inferred Projects**: In your plugin's `on_file_found()` implementation:
///
/// ```rust,no_run
/// # use marty_plugin_protocol::{InferredProject, Workspace, WorkspaceProvider};
/// # use std::path::Path;
/// # struct MyProvider;
/// # impl WorkspaceProvider for MyProvider {
/// # fn include_path_globs(&self) -> Vec<String> { vec![] }
/// fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
///     // Parse your config file...
///     let project_dir = path.parent()?.to_path_buf();
///     let name = /* extract from config */
/// #   String::new();
///     let workspace_dependencies = /* detect internal deps */
/// #   vec![];
///     
///     Some(InferredProject {
///         name,
///         project_dir,
///         discovered_by: "my-plugin".to_string(),
///         workspace_dependencies,
///     })
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredProject {
    /// The name of the project, typically extracted from the framework's config file.
    ///
    /// **Examples**:
    /// - For NPM/PNPM: The `name` field from package.json
    /// - For Cargo: The package name from Cargo.toml `[package]` section
    /// - For Python: The project name from pyproject.toml or setup.py
    pub name: String,

    /// Absolute path to the project directory.
    ///
    /// This is typically the parent directory of the configuration file that was detected.
    /// For example, if you found `/workspace/packages/lib-a/package.json`, this would be
    /// `/workspace/packages/lib-a`.
    pub project_dir: PathBuf,

    /// The plugin key that discovered this project.
    ///
    /// This should match the value returned by your plugin's `key()` method.
    /// Used for debugging and tracking which plugin found which projects.
    pub discovered_by: String,

    /// **CRITICAL**: List of OTHER workspace projects that this project depends on.
    ///
    /// ⚠️ **Important**: This should ONLY contain names of projects within the same workspace,
    /// NOT external packages from npm, crates.io, PyPI, etc.
    ///
    /// **Purpose**: Used for:
    /// - Determining build order (dependencies built first)
    /// - Impact analysis (which projects are affected by changes)
    /// - Generating dependency graphs
    ///
    /// **How to Detect Workspace Dependencies**:
    ///
    /// Different ecosystems have different conventions for referencing workspace projects:
    ///
    /// - **JavaScript/TypeScript (package.json)**:
    ///   - `"workspace:*"` protocol (PNPM)
    ///   - `"file:../other-package"` references
    ///   - Matching names in workspace packages
    ///
    /// - **Rust (Cargo.toml)**:
    ///   - `path = "../other-crate"` dependencies
    ///   - Workspace member references
    ///
    /// - **Python (pyproject.toml)**:
    ///   - `file:../other-package` in dependencies
    ///
    /// **Example**:
    /// ```rust
    /// # use marty_plugin_protocol::InferredProject;
    /// # use std::path::PathBuf;
    /// let project = InferredProject {
    ///     name: "my-api".to_string(),
    ///     project_dir: PathBuf::from("/workspace/packages/api"),
    ///     discovered_by: "pnpm".to_string(),
    ///     workspace_dependencies: vec![
    ///         "shared-types".to_string(),    // ✅ Another workspace project
    ///         "utils".to_string(),           // ✅ Another workspace project
    ///         // NOT "express", "lodash", etc. - those are external dependencies
    ///     ],
    /// };
    /// ```
    #[serde(default)]
    pub workspace_dependencies: Vec<String>,
}

/// Represents the entire workspace context provided to plugins during discovery.
///
/// **When Plugins Receive This**: The `Workspace` is passed to your plugin's
/// `on_file_found()` method, allowing you to:
/// - Check for existing projects (avoid duplicates)
/// - Resolve workspace dependencies
/// - Access the workspace root path
///
/// **Usage Example**:
/// ```rust,no_run
/// # use marty_plugin_protocol::{Workspace, InferredProject, WorkspaceProvider};
/// # use std::path::Path;
/// # struct MyProvider;
/// # impl WorkspaceProvider for MyProvider {
/// # fn include_path_globs(&self) -> Vec<String> { vec![] }
/// fn on_file_found(&self, workspace: &Workspace, path: &Path) -> Option<InferredProject> {
///     // Check if a project with this name already exists
///     let project_name = /* extract name */
/// #   String::new();
///     
///     let already_exists = workspace.projects.iter().any(|p| p.name == project_name)
///         || workspace.inferred_projects.iter().any(|p| p.name == project_name);
///     
///     if already_exists {
///         return None; // Don't create duplicate
///     }
///     
///     // Create the project...
/// #   None
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Absolute path to the workspace root directory.
    ///
    /// This is typically the directory containing the `.marty` configuration folder.
    pub root: PathBuf,

    /// Explicit projects with marty.yml/marty.yaml configuration files.
    ///
    /// These take precedence over inferred projects if both exist for the same directory.
    #[serde(default)]
    pub projects: Vec<Project>,

    /// Projects discovered automatically by plugins.
    ///
    /// Plugins add to this list by returning `Some(InferredProject)` from `on_file_found()`.
    /// Note: During plugin execution, this list grows as plugins discover more projects.
    #[serde(default)]
    pub inferred_projects: Vec<InferredProject>,
}

/// Type-safe identifier for plugins.
///
/// **Purpose**: Ensures plugin keys don't contain whitespace or invalid characters.
/// The key is used throughout Marty for:
/// - Plugin registration and loading
/// - Configuration file sections
/// - The `discovered_by` field in `InferredProject`
///
/// **Requirements**:
/// - No whitespace characters (spaces, tabs, newlines)
/// - Use kebab-case by convention: `"my-plugin"`, not `"my_plugin"` or `"MyPlugin"`
/// - Should be stable across plugin versions
///
/// **Example**:
/// ```rust
/// # use marty_plugin_protocol::PluginKey;
/// // Valid keys
/// let key1 = PluginKey::new("cargo").unwrap();
/// let key2 = PluginKey::new("typescript").unwrap();
/// let key3 = PluginKey::new("next-js").unwrap();
///
/// // Invalid keys (contain whitespace)
/// assert!(PluginKey::new("my plugin").is_err());
/// assert!(PluginKey::new("my\tplugin").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginKey(String);

impl PluginKey {
    /// Create a new `PluginKey` from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains whitespace characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::PluginKey;
    /// let key = PluginKey::new("cargo").unwrap();
    /// assert_eq!(key.as_str(), "cargo");
    ///
    /// // Whitespace is not allowed
    /// assert!(PluginKey::new("my plugin").is_err());
    /// ```
    pub fn new(key: impl Into<String>) -> Result<Self, String> {
        let key = key.into();
        if key.chars().any(char::is_whitespace) {
            return Err(format!(
                "Plugin key '{}' contains whitespace characters",
                key
            ));
        }
        Ok(Self(key))
    }

    /// Get the key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PluginKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
