# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup with core workspace management functionality
- Plugin-based architecture with WASM runtime
- CLI interface for workspace operations
- Automated release pipeline with cargo-dist
- Cross-platform binary distribution

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0] - 2025-10-01

### Added
- Initial release of Marty monorepo management tool
- Core workspace detection and traversal
- Task execution with dependency resolution
- Plugin system for language-specific workspace providers
- CLI commands: `list`, `deps`, `run`, `plan`
- Support for Cargo, PNPM, and TypeScript workspaces
- Comprehensive test suite and documentation

[Unreleased]: https://github.com/codyspate/marty/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/codyspate/marty/releases/tag/v0.1.0