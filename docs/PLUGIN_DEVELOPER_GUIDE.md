# Plugin Developer Guide

This guide is for developers creating Marty plugins. It covers the complete workflow from development to publication, including validation, testing, and release management.

## Quick Start

Marty provides CLI commands to assist with plugin development:

```bash
# Validate a plugin binary
marty plugin validate path/to/plugin.so --name typescript

# Generate release instructions
marty plugin release-guide typescript 0.2.0 --github-repo owner/repo --monorepo

# Check if a release is properly configured
marty plugin check-release --github-repo owner/repo --plugin typescript --version 0.2.0
```

## Plugin Development Workflow

### 1. Create Your Plugin

Follow the Marty plugin protocol to implement your plugin. Key requirements:

- Implement the `MartyPlugin` trait
- The `name()` method **must** return the plugin name you'll use in releases
- Export the plugin using the proper FFI interface

Example:
```rust
use marty_plugin_protocol::{MartyPlugin, Project, Dependency};

pub struct TypeScriptPlugin;

impl MartyPlugin for TypeScriptPlugin {
    fn name(&self) -> String {
        // ⚠️ CRITICAL: This must match your plugin name in releases
        "typescript".to_string()
    }

    fn discover_projects(&self, root: &Path) -> Vec<Project> {
        // Your implementation
    }

    fn discover_dependencies(&self, project: &Project) -> Vec<Dependency> {
        // Your implementation
    }
}

// FFI exports
#[no_mangle]
pub extern "C" fn marty_plugin_create() -> *mut dyn MartyPlugin {
    Box::into_raw(Box::new(TypeScriptPlugin))
}
```

### 2. Build and Test Locally

Build your plugin:
```bash
cargo build --release
```

The binary will be at:
- **Linux/macOS**: `target/release/libmarty_plugin_yourname.so` or `.dylib`
- **Windows**: `target/release/marty_plugin_yourname.dll`

Test it locally in a workspace:
```yaml
# .marty/workspace.yml
plugins:
  - path: "/absolute/path/to/target/release/libmarty_plugin_yourname.so"
```

### 3. Validate Your Plugin

Use the `validate` command to check your plugin before release:

```bash
marty plugin validate target/release/libmarty_plugin_typescript.so --name typescript
```

**Output:**
```
🔍 Validating plugin: target/release/libmarty_plugin_typescript.so

✅ File extension matches platform: .so

🔌 Loading plugin to check name...
   To fully validate, ensure your plugin's MartyPlugin::name() method returns the expected name.

📝 Expected plugin name: typescript
   ⚠️  Make sure MartyPlugin::name() returns exactly: "typescript"

📋 Binary Naming Convention Checklist:
   For release, your binary should be named:
   marty-plugin-typescript-v{VERSION}-x86_64-unknown-linux-gnu.so

✅ Basic validation complete!
```

**What it checks:**
- ✅ File exists and is readable
- ✅ File extension matches current platform (`.so`, `.dylib`, or `.dll`)
- ✅ Filename follows conventions (if applicable)
- ⚠️  Warns about naming mismatches

### 4. Build for All Platforms

To support all users, build for all supported platforms:

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM64 (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc

# Windows ARM64
cargo build --release --target aarch64-pc-windows-msvc
```

**Setting up cross-compilation:**

Install targets:
```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
```

For cross-platform builds, consider using:
- [cross](https://github.com/cross-rs/cross) for Linux/Windows targets
- macOS: Build on macOS or use GitHub Actions
- CI/CD: GitHub Actions can build all platforms

### 5. Generate Release Instructions

Use the `release-guide` command to get detailed release instructions:

```bash
# For monorepo (multiple plugins in one repo)
marty plugin release-guide typescript 0.2.0 --github-repo codyspate/marty --monorepo

# For separate repo (one plugin per repo)
marty plugin release-guide typescript 0.2.0 --github-repo codyspate/marty-plugin-typescript
```

**Output includes:**
1. Build commands for all platforms
2. Binary renaming commands
3. GitHub release creation commands
4. Example user configuration
5. Validation command

### 6. Rename Binaries for Release

Binaries must follow this naming convention:
```
marty-plugin-{name}-v{version}-{target}.{ext}
```

**Examples:**
```bash
# Linux x86_64
mv target/x86_64-unknown-linux-gnu/release/libmarty_plugin_typescript.so \
   marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so

# macOS ARM64
mv target/aarch64-apple-darwin/release/libmarty_plugin_typescript.dylib \
   marty-plugin-typescript-v0.2.0-aarch64-apple-darwin.dylib

# Windows x86_64
mv target/x86_64-pc-windows-msvc/release/marty_plugin_typescript.dll \
   marty-plugin-typescript-v0.2.0-x86_64-pc-windows-msvc.dll
```

### 7. Create GitHub Release

#### Option A: Monorepo Release

For multiple plugins in one repository:

**Tag format:** `marty-plugin-{name}-v{version}`

```bash
gh release create marty-plugin-typescript-v0.2.0 \
  marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so \
  marty-plugin-typescript-v0.2.0-aarch64-unknown-linux-gnu.so \
  marty-plugin-typescript-v0.2.0-x86_64-apple-darwin.dylib \
  marty-plugin-typescript-v0.2.0-aarch64-apple-darwin.dylib \
  marty-plugin-typescript-v0.2.0-x86_64-pc-windows-msvc.dll \
  marty-plugin-typescript-v0.2.0-aarch64-pc-windows-msvc.dll \
  --title "TypeScript Plugin v0.2.0" \
  --notes "Release notes here"
```

**User configuration:**
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.0
```

#### Option B: Separate Repository Release

For one plugin per repository:

**Tag format:** `v{version}`

**Repository name:** Must be `marty-plugin-{name}` (e.g., `marty-plugin-typescript`)

```bash
gh release create v0.2.0 \
  marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so \
  marty-plugin-typescript-v0.2.0-aarch64-unknown-linux-gnu.so \
  marty-plugin-typescript-v0.2.0-x86_64-apple-darwin.dylib \
  marty-plugin-typescript-v0.2.0-aarch64-apple-darwin.dylib \
  marty-plugin-typescript-v0.2.0-x86_64-pc-windows-msvc.dll \
  marty-plugin-typescript-v0.2.0-aarch64-pc-windows-msvc.dll \
  --title "v0.2.0" \
  --notes "Release notes here"
```

**User configuration:**
```yaml
plugins:
  - githubRepo: codyspate/marty-plugin-typescript
    version: 0.2.0
```

### 8. Validate the Release

After publishing, verify all platforms are available:

```bash
# Monorepo
marty plugin check-release \
  --github-repo codyspate/marty \
  --plugin typescript \
  --version 0.2.0

# Separate repo
marty plugin check-release \
  --github-repo codyspate/marty-plugin-typescript \
  --version 0.2.0
```

**Output:**
```
🔍 Checking GitHub release...

📦 Repository: codyspate/marty
🏷️  Release tag: marty-plugin-typescript-v0.2.0
📋 Plugin name: typescript

Checking for binaries on all platforms:

  ✅ Linux x86_64 - marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so
  ✅ Linux ARM64 - marty-plugin-typescript-v0.2.0-aarch64-unknown-linux-gnu.so
  ✅ macOS x86_64 - marty-plugin-typescript-v0.2.0-x86_64-apple-darwin.dylib
  ✅ macOS ARM64 - marty-plugin-typescript-v0.2.0-aarch64-apple-darwin.dylib
  ✅ Windows x86_64 - marty-plugin-typescript-v0.2.0-x86_64-pc-windows-msvc.dll
  ✅ Windows ARM64 - marty-plugin-typescript-v0.2.0-aarch64-pc-windows-msvc.dll

📊 Summary: 6/6 platforms available
✅ All platforms available! Plugin is ready for use.
```

## Common Issues and Solutions

### ❌ Plugin Name Mismatch

**Problem:** Plugin's `name()` method returns different name than expected.

**Solution:**
```rust
impl MartyPlugin for YourPlugin {
    fn name(&self) -> String {
        // Must match the name in your release tags and configuration
        "typescript".to_string() // Not "TypeScript" or "ts"!
    }
}
```

### ❌ Binary Naming Incorrect

**Problem:** Binaries don't follow the naming convention.

**Solution:** Use exact format:
```
marty-plugin-{name}-v{version}-{target}.{ext}
```

**Correct:**
- `marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so` ✅

**Incorrect:**
- `typescript-v0.2.0-x86_64-unknown-linux-gnu.so` ❌ (missing `marty-plugin-`)
- `marty-plugin-typescript-0.2.0-x86_64-unknown-linux-gnu.so` ❌ (missing `v` prefix)
- `marty-plugin-typescript-v0.2.0-linux-x64.so` ❌ (wrong target format)

### ❌ Release Tag Format Wrong

**Problem:** Tag doesn't match expected format.

**Solution:**

**Monorepo:** `marty-plugin-{name}-v{version}`
- ✅ `marty-plugin-typescript-v0.2.0`
- ❌ `typescript-v0.2.0`
- ❌ `v0.2.0`

**Separate Repo:** `v{version}`
- ✅ `v0.2.0`
- ❌ `0.2.0`
- ❌ `typescript-v0.2.0`

### ❌ Missing Platform Binaries

**Problem:** Not all platforms have binaries in the release.

**Impact:** Users on missing platforms can't use your plugin.

**Solution:** Build and upload all 6 platform binaries. Use GitHub Actions for automated builds:

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'marty-plugin-*-v*'
      - 'v*'

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
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
          
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
        
      - name: Rename binary
        run: |
          # Extract plugin name and version from tag
          TAG=${GITHUB_REF#refs/tags/}
          
          # Determine source path based on OS
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            SRC="target/${{ matrix.target }}/release/marty_plugin_yourname.${{ matrix.ext }}"
          else
            SRC="target/${{ matrix.target }}/release/libmarty_plugin_yourname.${{ matrix.ext }}"
          fi
          
          DEST="marty-plugin-yourname-${TAG#v}-${{ matrix.target }}.${{ matrix.ext }}"
          mv $SRC $DEST
          
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: marty-plugin-yourname-*.${{ matrix.ext }}
```

## Best Practices

### 1. Consistent Naming

- **Plugin name in code** = **Plugin name in config** = **Plugin name in releases**
- Use lowercase, hyphen-separated names (e.g., `typescript`, `pnpm`, `cargo`)
- Don't use underscores in public-facing names

### 2. Semantic Versioning

Follow [SemVer](https://semver.org/):
- **MAJOR**: Breaking changes to plugin API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### 3. Cross-Platform Testing

Test on multiple platforms before release:
- Use GitHub Actions for automated cross-platform builds
- Test with real workspaces on different OSes
- Validate all binaries exist before announcing release

### 4. Documentation

Provide clear documentation:
- README with plugin purpose and features
- Configuration examples
- Supported project types
- Known limitations

### 5. Changelog

Maintain a CHANGELOG.md:
```markdown
# Changelog

## [0.2.0] - 2025-10-03

### Added
- Support for workspace dependencies
- Improved error messages

### Fixed
- Bug in path resolution on Windows

## [0.1.0] - 2025-09-15

### Added
- Initial release
- Basic project discovery
```

## CLI Command Reference

### `marty plugin validate`

Validate a plugin binary before release.

```bash
marty plugin validate <PATH> [--name <NAME>]
```

**Arguments:**
- `<PATH>`: Path to the plugin binary
- `--name`: Expected plugin name (optional, for additional validation)

**Example:**
```bash
marty plugin validate target/release/libmarty_plugin_typescript.so --name typescript
```

### `marty plugin release-guide`

Generate comprehensive release instructions.

```bash
marty plugin release-guide <NAME> <VERSION> [OPTIONS]
```

**Arguments:**
- `<NAME>`: Plugin name (e.g., `typescript`)
- `<VERSION>`: Version (e.g., `0.2.0`)

**Options:**
- `--github-repo <REPO>`: GitHub repository (e.g., `owner/repo`)
- `--monorepo`: Use monorepo release format

**Example:**
```bash
marty plugin release-guide typescript 0.2.0 \
  --github-repo codyspate/marty \
  --monorepo
```

### `marty plugin check-release`

Verify a GitHub release has all platform binaries.

```bash
marty plugin check-release [OPTIONS]
```

**Options:**
- `--github-repo <REPO>`: GitHub repository (required)
- `--plugin <NAME>`: Plugin name (for monorepo releases)
- `--version <VERSION>`: Version to check (required)

**Example:**
```bash
# Monorepo
marty plugin check-release \
  --github-repo codyspate/marty \
  --plugin typescript \
  --version 0.2.0

# Separate repo
marty plugin check-release \
  --github-repo codyspate/marty-plugin-typescript \
  --version 0.2.0
```

## Platform Reference

| Platform | Target Triple | Extension | Build Command |
|----------|---------------|-----------|---------------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.so` | `cargo build --release --target x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.so` | `cargo build --release --target aarch64-unknown-linux-gnu` |
| macOS x86_64 | `x86_64-apple-darwin` | `.dylib` | `cargo build --release --target x86_64-apple-darwin` |
| macOS ARM64 | `aarch64-apple-darwin` | `.dylib` | `cargo build --release --target aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.dll` | `cargo build --release --target x86_64-pc-windows-msvc` |
| Windows ARM64 | `aarch64-pc-windows-msvc` | `.dll` | `cargo build --release --target aarch64-pc-windows-msvc` |

## See Also

- [Plugin Monorepo Approach](./PLUGIN_MONOREPO_APPROACH.md) - Detailed monorepo documentation
- [Plugin Resolution](./PLUGIN_RESOLUTION.md) - How Marty resolves plugins
- [Plugin Protocol](../crates/plugin_protocol/README.md) - Plugin API reference
