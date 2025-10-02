//! Command execution utilities
//! 
//! This module provides a unified interface for executing different types of commands
//! (shell commands, scripts, executable with args) with consistent error handling and logging.

use std::path::PathBuf;
use std::process::Command;

use colored::*;

use crate::tasks::get_project_color;
use crate::types::{MartyError, MartyResult};
use crate::workspace::Workspace;

/// Unified command executor that handles common setup and execution patterns
pub struct CommandExecutor<'a> {
    workspace: &'a Workspace,
    targets: &'a [String],
}

impl<'a> CommandExecutor<'a> {
    pub fn new(workspace: &'a Workspace, targets: &'a [String]) -> Self {
        Self { workspace, targets }
    }

    /// Execute a command with common setup and error handling
    pub fn execute_command(
        &self,
        command: &mut Command,
        execution_error_message: &str,
        failure_error_message: &str,
    ) -> MartyResult<()> {
        // Common setup
        command.current_dir(&self.workspace.root);
        
        // Set environment variables for targets
        for (i, target) in self.targets.iter().enumerate() {
            command.env(format!("MARTY_TARGET_{}", i), target);
        }

        // Execute command
        let status = command
            .status()
            .map_err(|e| MartyError::Task(format!("{}: {}", execution_error_message, e)))?;

        if !status.success() {
            return Err(MartyError::Task(format!(
                "{}: {}",
                failure_error_message,
                status.code().unwrap_or(-1)
            )));
        }

        self.show_completion_message();
        Ok(())
    }

    /// Execute a script file
    pub fn execute_script(&self, script_path: &str) -> MartyResult<()> {
        let script_path_buf = PathBuf::from(script_path);

        // If script path is relative, resolve it relative to workspace root
        let full_script_path = if script_path_buf.is_relative() {
            self.workspace.root.join(script_path_buf)
        } else {
            script_path_buf
        };

        if !full_script_path.exists() {
            return Err(MartyError::Task(format!(
                "Script file '{}' not found",
                full_script_path.display()
            )));
        }

        let mut command = Command::new(&full_script_path);
        self.execute_command(
            &mut command,
            &format!("Failed to execute script: {}", full_script_path.display()),
            "Script execution failed with exit code",
        )
    }

    /// Execute a command with arguments
    pub fn execute_command_with_args(&self, command_path: &str, args: &[String]) -> MartyResult<()> {
        let mut command = Command::new(command_path);
        command.args(args);
        self.execute_command(
            &mut command,
            &format!("Failed to execute command '{}'", command_path),
            &format!("Command '{}' failed with exit code", command_path),
        )
    }

    /// Execute a single shell command
    pub fn execute_shell_command(&self, cmd: &str) -> MartyResult<()> {
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd);
        self.execute_command(
            &mut command,
            &format!("Failed to execute command '{}'", cmd),
            &format!("Command '{}' failed with exit code", cmd),
        )
    }

    /// Show completion message for the first target
    fn show_completion_message(&self) {
        if let Some(target) = self.targets.first() {
            let project_color = get_project_color(target);
            println!(
                "{} {}",
                "✓".green().bold(),
                format!("Completed for {}", target).color(project_color)
            );
        }
    }
}