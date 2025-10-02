# Marty

<div align="center">
  <img src="docs/images/marty.png" alt="Marty Logo" width="200" height="200">
</div>

## Overview

Marty is a language-agnostic monorepo management tool built in Rust, designed to simplify workspace management and task execution across multi-language projects. With its plugin-based architecture and WASM runtime, Marty provides flexible workspace detection, dependency resolution, and task orchestration.

## Features

- **Language Agnostic**: Supports any programming language through plugins
- **Plugin Architecture**: WASM-based plugins for flexible workspace providers
- **Dependency Resolution**: Intelligent task dependency management and parallel execution
- **Workspace Detection**: Automatic project discovery and workspace configuration
- **Task Orchestration**: Define and execute complex build pipelines across projects
- **Type Safety**: Built with Rust's type system for reliability and performance

## Architecture

Marty is organized as a Cargo workspace with the following crates:

- **`marty_core`**: Core business logic, workspace management, and execution engine
- **`marty_cli`**: Command-line interface for user interactions
- **`plugin_protocol`**: Protocol definitions for WASM plugin communication
- **`plugins/`**: Collection of workspace provider plugins (cargo, etc.)

### Core Components

- **WorkspaceManager**: High-level interface for all workspace operations
- **Execution Engine**: Modular task execution with command handling and dependency resolution
- **Plugin Runtime**: WASM-based plugin system for extensible workspace providers
- **Task Runner**: Parallel execution coordinator with dependency management

## Quick Start

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd marty

# Build the project
cargo build --release

# Run tests
cargo test

# Install locally (optional)
cargo install --path crates/cli
```

### Basic Usage

```bash
# Initialize a workspace
marty init

# List projects in workspace
marty list

# Show project dependencies
marty deps

# Run tasks on specific projects
marty run build --target my-project

# Execute tasks with dependencies
marty plan --target my-project --task test
```

## Configuration

Marty uses YAML configuration files for workspace and task definitions:

### Workspace Configuration (`.marty/workspace.yml`)

```yaml
name: "My Workspace"
plugins:
  - name: "cargo"
    config:
      includes: ["crates/**", "plugins/*"]
```

### Task Definitions (`.marty/tasks/build.yml`)

```yaml
name: "Build Tasks"
description: "Common build tasks for the workspace"
tags: 
  - rust
tasks:
  - name: "build"
    description: "Build all projects"
    command: ["cargo", "build"]
  - name: "test"
    description: "Run tests for all projects"
    command: ["cargo", "test"]
    dependencies: ["build"]
```

## Plugin Development

Marty's plugin system uses WASM for safe, portable extensions. Plugins implement workspace providers for different project types and languages.

### Core Plugin Interface

```rust
use plugin_protocol::{WorkspaceProvider, ProjectInfo};

// Plugins implement the WorkspaceProvider trait
// and are compiled to WASM modules
```

## Project Structure

```
marty/
├── crates/
│   ├── core/           # Core business logic
│   └── cli/            # Command-line interface
├── plugins/            # WASM workspace providers
├── plugin_protocol/    # Plugin communication protocol
├── examples/          # Example workspaces and usage
├── docs/             # Documentation and assets
└── .marty/           # Workspace configuration
```

## Contributing

We welcome contributions! Please ensure:

1. All code follows Rust best practices and passes `cargo clippy`
2. Tests are included for new features
3. Documentation is updated for API changes
4. Commits are atomic and well-described

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-flamegraph

# Run tests in watch mode
cargo watch -x test

# Check lints
cargo clippy --all-targets --all-features
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Cody Spate** - Creator and maintainer

---

Built with ❤️ in Rust for the monorepo management community.