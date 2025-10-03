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

# Plugin management
marty plugin list           # List cached plugins
marty plugin clear          # Clear plugin cache
marty plugin update         # Update all plugins from URLs
```

## Configuration

Marty uses YAML configuration files for workspace and task definitions:

### Workspace Configuration (`.marty/workspace.yml`)

```yaml
name: "My Workspace"
plugins:
  # GitHub convention (recommended) - automatically resolves platform-specific binaries
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      includes: ["crates/**", "plugins/*"]
  
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    options:
      auto_project_references: true
      
  # Direct URL for custom hosting (fallback)
  - url: "https://custom-host.com/plugins/my-plugin.so"
    options:
      custom_option: true
      
  # Local file path for development
  - path: "/path/to/custom-plugin.so"
    enabled: false
```

**Plugin Resolution:**
- **GitHub Convention**: Just specify `repository` + `version`, Marty automatically downloads the correct binary for your platform
- **Direct URL**: Specify exact URL to plugin binary (not cross-platform)
- **Local Path**: Use local filesystem path for development

See [Plugin Resolution Guide](docs/PLUGIN_RESOLUTION.md) for details.

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

## Plugin System

Marty's plugin system uses WASM for safe, portable extensions. Plugins implement workspace providers for different project types and languages.

### Plugin Configuration

Plugins can be configured in three ways:

1. **Built-in plugins** - Use `path: "builtin"` for plugins bundled with Marty
2. **URL-based plugins** - Download plugins from URLs and cache them locally
3. **Local file plugins** - Point to local `.wasm` files

### Plugin Options

Each plugin can have custom configuration through the `options` field:

```yaml
plugins:
  - name: "cargo"
    path: "builtin"
    options:
      includes: ["crates/**"]
      excludes: ["target/**"]
      
  - name: "typescript"
    url: "https://example.com/typescript-plugin.wasm"
    options:
      compilerOptions:
        strict: true
        target: "ES2020"
```

### Plugin Caching

URL-based plugins are automatically downloaded and cached in `.marty/cache/plugins/`. The cache uses URL hashing to avoid re-downloading unchanged plugins.

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

## Releases

Marty uses [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated binary releases and [cargo-release](https://github.com/crate-ci/cargo-release) for version management.

### Release Process

The recommended workflow uses pull requests:

```bash
# 1. Create a release branch
git checkout -b release-v0.2.0

# 2. Update changelog and other release preparation
# Edit CHANGELOG.md, update version references, etc.
git commit -am "prep release v0.2.0"

# 3. Use cargo-release to update versions and push
cargo release --no-publish --no-tag --allow-branch=release-v0.2.0 0.2.0

# 4. Create PR, review, and merge to main

# 5. Complete the release from main branch
git checkout main
git pull
cargo release  # This creates the tag and triggers CI
```

### What Happens During Release

1. **cargo-release** handles version bumping across the workspace
2. **GitHub Actions** automatically builds binaries for multiple platforms
3. **GitHub Release** is created with downloadable assets
4. **Installers** are generated (shell script, PowerShell script)
5. **Checksums** are provided for all artifacts

### Installation Methods

Users can install Marty in several ways:

```bash
# Via shell installer (macOS/Linux)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/codyspate/marty/releases/latest/download/marty_cli-installer.sh | sh

# Via PowerShell installer (Windows)
powershell -c "irm https://github.com/codyspate/marty/releases/latest/download/marty_cli-installer.ps1 | iex"

# Manual download from GitHub Releases
# Download the appropriate archive for your platform
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