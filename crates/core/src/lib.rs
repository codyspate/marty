//! Marty Core Library
//!
//! This is the core library for the Marty monorepo management tool. It provides
//! all the business logic for workspace management, task execution, plugin handling,
//! and project discovery.
//!
//! ## Architecture
//!
//! The core library is organized into several modules:
//!
//! - [`workspace_manager`] - High-level workspace management interface
//! - [`execution`] - Task execution engine with dependency resolution
//! - [`workspace`] - Low-level workspace operations and discovery
//! - [`task_execution`] - Task execution planning and compatibility checking
//! - [`tasks`] - Task utilities and color management
//! - [`configs`] - Configuration parsing for workspace, projects, and tasks
//! - [`plugin_runtime`] - WASM plugin runtime for workspace providers
//! - [`results`] - Result types for workspace operations
//! - [`types`] - Common error types and type aliases
//!
//! ## Usage
//!
//! The primary entry point is the [`WorkspaceManager`] which provides a high-level
//! interface for all workspace operations:
//!
//! ```rust,no_run
//! use marty_core::workspace_manager::{WorkspaceManager, WorkspaceManagerConfig};
//! use std::path::PathBuf;
//!
//! # async fn example() -> marty_core::types::MartyResult<()> {
//! let manager = WorkspaceManager::new(WorkspaceManagerConfig {
//!     workspace_root: PathBuf::from("."),
//! }).await?;
//!
//! let projects = manager.list_projects(false)?;
//! # Ok(())
//! # }
//! ```

pub mod configs;
pub mod execution;
pub mod platform;
pub mod plugin_cache;
pub mod plugin_runtime_dylib;
pub mod results;
pub mod task_execution;
pub mod tasks;
pub mod types;
pub mod workspace;
pub mod workspace_manager;

// Re-export the main types for easier usage
pub use types::{MartyError, MartyResult};
pub use workspace_manager::{WorkspaceManager, WorkspaceManagerConfig};
