//! Serializable message types for plugin communication.
//!
//! This module contains types used for cross-boundary communication between
//! Marty and plugins, especially for FFI/dynamic library interfaces.

use crate::types::InferredProject;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Serializable version of [`InferredProject`] for plugin communication.
///
/// **Purpose**: This type is used for passing project information across the FFI boundary
/// between Marty and dynamically loaded plugins. It uses `String` instead of `PathBuf`
/// for maximum compatibility with C ABIs.
///
/// **When to Use**:
/// - Implementing the C ABI layer in plugins
/// - Serializing project data to JSON
/// - Communicating across process boundaries
///
/// **Conversion**: Easily converts to/from `InferredProject`:
///
/// ```rust
/// # use marty_plugin_protocol::{InferredProject, InferredProjectMessage};
/// # use std::path::PathBuf;
/// // InferredProject -> InferredProjectMessage
/// let project = InferredProject {
///     name: "my-project".to_string(),
///     project_dir: PathBuf::from("/workspace/project"),
///     discovered_by: "pnpm".to_string(),
///     workspace_dependencies: vec![],
/// };
/// let message: InferredProjectMessage = project.into();
///
/// // InferredProjectMessage -> InferredProject
/// let project_back: InferredProject = message.into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredProjectMessage {
    /// The name of the project.
    pub name: String,

    /// Project directory path as a string (instead of PathBuf for FFI compatibility).
    pub project_dir: String,

    /// The plugin key that discovered this project.
    pub discovered_by: String,

    /// List of workspace project dependencies.
    pub workspace_dependencies: Vec<String>,
}

impl InferredProjectMessage {
    /// Create a new `InferredProjectMessage`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use marty_plugin_protocol::InferredProjectMessage;
    /// let message = InferredProjectMessage::new(
    ///     "my-api",
    ///     "/workspace/packages/api",
    ///     "pnpm",
    ///     vec!["shared-types".to_string()],
    /// );
    /// ```
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
