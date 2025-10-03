# Plugin Monorepo Approach

This guide explains how to configure plugins when they are published as releases in a monorepo (like the main `marty` repository) rather than in separate plugin-specific repositories.

## Overview

Marty supports two approaches for plugin distribution via GitHub:

1. **Separate Repository Approach**: Each plugin has its own repository (e.g., `marty-plugin-typescript`)
2. **Monorepo Approach**: Multiple plugins are published as releases in a single repository (e.g., `marty`)

## Monorepo Configuration

When using the monorepo approach, you specify:
- `repository`: The GitHub repository containing the plugin releases
- `plugin`: The plugin name (used to find the correct release)
- `version`: The plugin version

### Example Configuration

```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
    
  - githubRepo: codyspate/marty
    plugin: pnpm
    version: 0.1.0
```

## How It Works

### Release Tag Format

For monorepo plugins, releases must be tagged with:
```
marty-plugin-{plugin-name}-v{version}
```

Examples:
- `marty-plugin-typescript-v0.2.2`
- `marty-plugin-pnpm-v0.1.0`
- `marty-plugin-cargo-v1.0.0`

### Binary Naming Convention

The binary assets in each release must follow this format:
```
marty-plugin-{plugin-name}-v{version}-{target}.{ext}
```

Examples:
- `marty-plugin-typescript-v0.2.2-x86_64-unknown-linux-gnu.so`
- `marty-plugin-typescript-v0.2.2-aarch64-apple-darwin.dylib`
- `marty-plugin-typescript-v0.2.2-x86_64-pc-windows-msvc.dll`

### URL Resolution

When you configure:
```yaml
githubRepo: codyspate/marty
plugin: typescript
version: 0.2.2
```

Marty automatically constructs the URL:
```
https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.2/marty-plugin-typescript-v0.2.2-{target}.{ext}
```

Where `{target}` and `{ext}` are determined by the user's platform.

## Comparison with Separate Repository Approach

### Separate Repository

```yaml
plugins:
  - githubRepo: codyspate/marty-plugin-typescript
    version: 0.2.2
```

- Release tag: `v0.2.2`
- URL: `https://github.com/codyspate/marty-plugin-typescript/releases/download/v0.2.2/marty-plugin-typescript-v0.2.2-{target}.{ext}`
- Plugin name extracted from repository name

### Monorepo

```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

- Release tag: `marty-plugin-typescript-v0.2.2`
- URL: `https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.2/marty-plugin-typescript-v0.2.2-{target}.{ext}`
- Plugin name explicitly specified

## Benefits of Monorepo Approach

1. **Centralized Management**: All plugins in one repository
2. **Coordinated Releases**: Release multiple plugins together
3. **Shared Infrastructure**: One CI/CD pipeline for all plugins
4. **Simpler Governance**: Single repo for issues, PRs, and discussions

## Publishing Plugins in a Monorepo

### 1. Build for All Platforms

Build your plugin for all supported platforms:
```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM64
cargo build --release --target aarch64-apple-darwin

# Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc

# Windows ARM64
cargo build --release --target aarch64-pc-windows-msvc
```

### 2. Rename Binaries

Rename each binary to follow the naming convention:
```bash
mv target/x86_64-unknown-linux-gnu/release/libmarty_plugin_typescript.so \
   marty-plugin-typescript-v0.2.2-x86_64-unknown-linux-gnu.so
```

### 3. Create GitHub Release

Create a release with the tag `marty-plugin-typescript-v0.2.2` and upload all platform binaries.

### 4. Example GitHub Actions Workflow

```yaml
name: Release Plugin

on:
  push:
    tags:
      - 'marty-plugin-*-v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            ext: so
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            ext: so
          - os: macos-latest
            target: x86_64-apple-darwin
            ext: dylib
          - os: macos-latest
            target: aarch64-apple-darwin
            ext: dylib
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            ext: dll
          - os: windows-latest
            target: aarch64-pc-windows-msvc
            ext: dll

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Extract plugin name and version
        id: extract
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          echo "tag=$TAG" >> $GITHUB_OUTPUT
          
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
        
      - name: Rename binary
        run: |
          # Determine library prefix/extension
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            BINARY="target/${{ matrix.target }}/release/marty_plugin_typescript.dll"
          elif [ "${{ matrix.os }}" = "macos-latest" ]; then
            BINARY="target/${{ matrix.target }}/release/libmarty_plugin_typescript.dylib"
          else
            BINARY="target/${{ matrix.target }}/release/libmarty_plugin_typescript.so"
          fi
          
          mv $BINARY ${{ steps.extract.outputs.tag }}-${{ matrix.target }}.${{ matrix.ext }}
          
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: ${{ steps.extract.outputs.tag }}-${{ matrix.target }}.${{ matrix.ext }}
```

## Platform Support

Marty automatically detects the user's platform and downloads the correct binary:

| Platform | Target Triple | Extension |
|----------|---------------|-----------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.so` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.so` |
| macOS x86_64 | `x86_64-apple-darwin` | `.dylib` |
| macOS ARM64 | `aarch64-apple-darwin` | `.dylib` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.dll` |
| Windows ARM64 | `aarch64-pc-windows-msvc` | `.dll` |

## Error Handling

If a plugin binary is not found for the user's platform, Marty provides helpful error messages:

```
Error: Failed to download plugin from https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.2/...

Your platform: x86_64-unknown-linux-gnu (.so)

Please check:
1. The release 'marty-plugin-typescript-v0.2.2' exists at: https://github.com/codyspate/marty/releases
2. The release contains the binary: marty-plugin-typescript-v0.2.2-x86_64-unknown-linux-gnu.so
3. Your repository is public or you have access to it

Alternative: Use direct URL if the plugin is hosted elsewhere:
  url: https://example.com/path/to/plugin.so
```

## Migration Guide

### From Direct URLs to Monorepo

Before:
```yaml
plugins:
  - url: https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.2/marty-plugin-typescript-v0.2.2-x86_64-unknown-linux-gnu.so
```

After:
```yaml
plugins:
  - repository: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

**Benefits**: Platform-agnostic configuration that works on Linux, macOS, and Windows.

### From Separate Repos to Monorepo

Before:
```yaml
plugins:
  - repository: codyspate/marty-plugin-typescript
    version: 0.2.2
```

After:
```yaml
plugins:
  - repository: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

**Required Changes**:
1. Update release tags to include plugin name
2. Update binary naming to include plugin name
3. Add `plugin` field to configurations

## Best Practices

1. **Consistent Versioning**: Version plugins independently or together based on your needs
2. **Changelog per Plugin**: Maintain separate changelogs for each plugin in the monorepo
3. **CI/CD**: Automate cross-platform builds and releases
4. **Testing**: Test plugins on all supported platforms before release
5. **Documentation**: Document each plugin's capabilities and configuration options

## See Also

- [Plugin Resolution Guide](./PLUGIN_RESOLUTION.md) - Complete plugin resolution documentation
- [Plugin Configuration Examples](./PLUGIN_CONFIGURATION_EXAMPLES.md) - Configuration patterns
- [Plugin Development Guide](./PLUGIN_DEVELOPMENT.md) - How to create plugins
