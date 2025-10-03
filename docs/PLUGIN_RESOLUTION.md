# Plugin Resolution Guide

This guide explains how Marty resolves and loads plugins across different platforms and hosting scenarios.

## Overview

Marty supports three plugin resolution strategies with automatic fallback:

1. **GitHub Convention** (Recommended) - Automatic platform-specific binary resolution
2. **Direct URL** - Explicit URL to a plugin binary
3. **Local Path** - Local filesystem path to a plugin binary

## Plugin Configuration Formats

### 1. GitHub Convention (Recommended)

The simplest and most cross-platform friendly approach:

```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      # Plugin-specific options
```

**How it works:**
- Marty detects your OS and architecture automatically
- Constructs the GitHub release URL following the standard naming convention
- Downloads and caches the appropriate binary

**URL Construction:**
```
https://github.com/{repository}/releases/download/v{version}/marty-plugin-{name}-v{version}-{target}.{ext}
```

**Example:**
- Repository: `codyspate/marty-plugin-cargo`
- Version: `0.2.0`
- Platform: Linux x86_64
- Generated URL: `https://github.com/codyspate/marty-plugin-cargo/releases/download/v0.2.0/marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so`

**Benefits:**
- ✅ Cross-platform: Same config works on all platforms
- ✅ Shareable: Team members on different OSes use same config
- ✅ Simple: Just specify repository and version
- ✅ Familiar: Uses GitHub releases (free hosting)

### 2. Direct URL

For custom hosting or non-standard naming:

```yaml
plugins:
  - url: "https://custom-host.com/plugins/my-plugin-v1.0.0-linux-x64.so"
    options:
      # Plugin-specific options
```

**Use cases:**
- Custom plugin hosting
- Non-standard binary names
- Private artifact servers
- Development/testing

**Limitations:**
- ❌ Not cross-platform: Different URLs needed for each platform
- ❌ Manual updates: Must update URL when version changes

### 3. Local Path

For local development or pre-installed plugins:

```yaml
plugins:
  - path: "/absolute/path/to/plugin.so"
    options:
      # Plugin-specific options
      
  # Special "builtin" path for plugins in .marty/plugins/
  - path: "builtin"
    options: {}
```

**Use cases:**
- Plugin development
- CI/CD with pre-installed plugins
- Offline/airgapped environments

## Supported Platforms

Marty automatically detects and resolves binaries for:

| Platform | Target Triple | Extension |
|----------|--------------|-----------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.so` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.so` |
| macOS x86_64 | `x86_64-apple-darwin` | `.dylib` |
| macOS ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `.dylib` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.dll` |
| Windows ARM64 | `aarch64-pc-windows-msvc` | `.dll` |

## Plugin Naming Convention

For GitHub convention to work, plugin repositories **must** follow this naming:

```
marty-plugin-{name}
```

**Examples:**
- ✅ `marty-plugin-cargo`
- ✅ `marty-plugin-typescript`
- ✅ `marty-plugin-pnpm`
- ❌ `cargo-plugin` (missing prefix)
- ❌ `marty-cargo` (wrong format)

## Plugin Caching

All downloaded plugins are cached in `.marty/cache/plugins/`:

```
.marty/
  cache/
    plugins/
      typescript_2da6c6fc.so
      cargo_4b8e9a12.dylib
```

**Cache Key:** Plugin name + URL hash (first 8 characters)

**Benefits:**
- Fast: Downloads happen once per URL
- Offline: Cached plugins work without network
- Clean: Separate cache per workspace

**Cache Management:**
```bash
# Clear plugin cache (future feature)
marty plugin cache clear

# List cached plugins (future feature)
marty plugin cache list
```

## Plugin Loading Process

1. **Parse Configuration**: Read workspace.yml plugin definitions
2. **Resolve URLs**: 
   - GitHub convention → construct platform-specific URL
   - Direct URL → use as-is
   - Local path → resolve filesystem path
3. **Download/Cache**: 
   - Check cache for existing binary
   - Download if not cached
   - Verify binary is valid dynamic library
4. **Load Plugin**: 
   - Load dynamic library
   - Extract plugin name from implementation
   - Validate configuration options
5. **Initialize**: Register plugin for workspace scanning

## Error Handling

### Plugin Not Found (404)

```
Error: Failed to download plugin from https://github.com/.../plugin.so: HTTP 404 Not Found
```

**Causes:**
- Plugin version doesn't exist
- Binary not published for your platform
- Repository or release deleted

**Solutions:**
- Check available versions/releases on GitHub
- Verify platform is supported
- Use direct URL as fallback

### Unsupported Platform

```
Error: Unsupported platform: freebsd-x86_64
```

**Solution:**
- Request platform support from plugin author
- Build plugin locally for your platform
- Use local path configuration

### Invalid Repository Format

```
Error: Repository name must start with 'marty-plugin-': 'cargo'
```

**Solution:**
- Ensure repository name follows `marty-plugin-{name}` convention
- Use `codyspate/marty-plugin-cargo`, not `codyspate/cargo`

### Configuration Validation Errors

```
Error: Plugin 'typescript' option 'invalid_option' not supported
```

**Solution:**
- Check plugin documentation for valid options
- Use `marty plugin info {name}` to see configuration schema (future feature)

## Best Practices

### For Plugin Users

1. **Use GitHub Convention**: Prefer `repository` + `version` over direct URLs
2. **Pin Versions**: Always specify exact versions, not ranges
3. **Document Options**: Comment plugin options in workspace.yml
4. **Test Offline**: Verify builds work with cached plugins

**Example:**
```yaml
plugins:
  # Cargo plugin for Rust workspace detection
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"  # Pinned version for reproducibility
    options:
      includes: ["crates/**"]
      excludes: ["**/target/**"]
```

### For Plugin Authors

1. **Follow Naming Convention**: Use `marty-plugin-{name}` for repositories
2. **Support All Platforms**: Build and publish binaries for all major platforms
3. **Use Standard Naming**: Follow the binary naming convention
4. **Provide Documentation**: Document all configuration options
5. **Use GitHub Actions**: Automate cross-platform builds and releases

**Example GitHub Actions Workflow:**
```yaml
name: Release Plugin
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            ext: so
          - os: macos-latest
            target: aarch64-apple-darwin
            ext: dylib
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            ext: dll
    
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - name: Build Plugin
        run: cargo build --release
        
      - name: Rename Binary
        run: |
          mv target/release/libmarty_plugin_*.${matrix.ext} \
             marty-plugin-name-${GITHUB_REF#refs/tags/v}-${{ matrix.target }}.${{ matrix.ext }}
      
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: marty-plugin-name-*.${matrix.ext}
```

## Migration Guide

### From Direct URLs to GitHub Convention

**Before:**
```yaml
plugins:
  - url: "https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.1/marty-plugin-typescript-v0.2.1-x86_64-unknown-linux-gnu.so"
    options:
      auto_project_references: true
```

**After:**
```yaml
plugins:
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    options:
      auto_project_references: true
```

**Benefits:**
- Works on all platforms automatically
- Shorter, cleaner configuration
- Easier version updates

## Future Enhancements

### Plugin Registry (Under Consideration)

```yaml
plugins:
  # Simple name resolution via registry
  - name: "cargo"
    version: "^0.2.0"  # Semantic versioning support
```

### Version Ranges (Under Consideration)

```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "^0.2.0"  # Compatible with 0.2.x
```

### Plugin Discovery Commands

```bash
# Search for plugins
marty plugin search typescript

# Show plugin information
marty plugin info cargo

# List installed plugins
marty plugin list

# Update all plugins
marty plugin update
```

## Troubleshooting

### Check Generated URL

Add debug logging to see what URL is being generated:

```bash
RUST_LOG=debug marty list
```

### Manual Download Test

Test if the URL is accessible:

```bash
# Example for cargo plugin on Linux
curl -I "https://github.com/codyspate/marty-plugin-cargo/releases/download/v0.2.0/marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so"
```

### Verify Platform Detection

Check your platform information:

```bash
rustc -vV | grep host
```

### Clear Cache

If experiencing issues with cached plugins:

```bash
rm -rf .marty/cache/plugins/
```

## Support

For plugin resolution issues:

1. Check this documentation
2. Verify plugin releases exist on GitHub
3. Test with direct URL as fallback
4. Report issues to plugin repository
5. For Marty core issues, file bug at https://github.com/codyspate/marty/issues
