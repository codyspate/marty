//! Task execution module
//!
//! This module handles the actual execution of tasks including command execution,
//! dependency management, and result reporting.

pub mod command;
pub mod dependencies;
pub mod runner;

pub use command::CommandExecutor;
pub use dependencies::group_by_dependency_levels;
pub use runner::{TaskRunner, TaskRunnerConfig};
