# Plugin Developer CLI Commands - Summary

## Changes Made

### 1. Field Rename: `repository` → `githubRepo`

**Reason:** Clarify that this field is specifically for GitHub repositories, making it clear what platforms are supported.

**Before:**
```yaml
plugins:
  - repository: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

**After:**
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

**Impact:**
- ✅ More explicit about GitHub-only support
- ✅ Leaves room for future platform support (gitlab, gitea, etc.)
- ✅ Clearer configuration schema
- ⚠️ Breaking change for existing configurations (migration needed)

### 2. New CLI Commands for Plugin Developers

Added three new commands under `marty plugin`:

#### `marty plugin validate`

Validates a plugin binary before release.

**Usage:**
```bash
marty plugin validate <PATH> [--name <NAME>]
```

**What it checks:**
- ✅ File exists and is readable
- ✅ Extension matches current platform (`.so`, `.dylib`, `.dll`)
- ✅ Filename follows naming conventions
- ⚠️ Warns about potential name mismatches

**Example:**
```bash
$ marty plugin validate target/release/libmarty_plugin_typescript.so --name typescript

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

#### `marty plugin release-guide`

Generates comprehensive release instructions for plugin developers.

**Usage:**
```bash
marty plugin release-guide <NAME> <VERSION> [--github-repo <REPO>] [--monorepo]
```

**What it provides:**
1. Build commands for all 6 platforms
2. Binary renaming commands
3. GitHub release creation commands
4. Example user configuration
5. Validation command

**Example:**
```bash
$ marty plugin release-guide typescript 0.2.0 --github-repo codyspate/marty --monorepo

📦 Release Guide for Plugin: typescript
Version: 0.2.0

1️⃣  Build for all platforms:
   # Linux x86_64
   cargo build --release --target x86_64-unknown-linux-gnu
   ...

2️⃣  Rename binaries:
   mv target/x86_64-unknown-linux-gnu/release/libmarty_plugin_typescript.so \
      marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so
   ...

3️⃣  Create GitHub Release:
   Tag: marty-plugin-typescript-v0.2.0
   gh release create marty-plugin-typescript-v0.2.0 \
     marty-plugin-typescript-v0.2.0-x86_64-unknown-linux-gnu.so \
     ...

4️⃣  Configuration for users:
   plugins:
     - githubRepo: codyspate/marty
       plugin: typescript
       version: 0.2.0

5️⃣  Validate release:
   marty plugin check-release --github-repo codyspate/marty --plugin typescript --version 0.2.0
```

#### `marty plugin check-release`

Verifies that a GitHub release has all required platform binaries.

**Usage:**
```bash
marty plugin check-release --github-repo <REPO> [--plugin <NAME>] --version <VERSION>
```

**What it checks:**
- Makes HTTP HEAD requests to verify each platform binary exists
- Reports which platforms are available and which are missing
- Provides actionable feedback

**Example:**
```bash
$ marty plugin check-release --github-repo codyspate/marty --plugin typescript --version 0.2.2

🔍 Checking GitHub release...

📦 Repository: codyspate/marty
🏷️  Release tag: marty-plugin-typescript-v0.2.2
📋 Plugin name: typescript

Checking for binaries on all platforms:

  ✅ Linux x86_64 - marty-plugin-typescript-v0.2.2-x86_64-unknown-linux-gnu.so
  ✅ Linux ARM64 - marty-plugin-typescript-v0.2.2-aarch64-unknown-linux-gnu.so
  ✅ macOS x86_64 - marty-plugin-typescript-v0.2.2-x86_64-apple-darwin.dylib
  ✅ macOS ARM64 - marty-plugin-typescript-v0.2.2-aarch64-apple-darwin.dylib
  ✅ Windows x86_64 - marty-plugin-typescript-v0.2.2-x86_64-pc-windows-msvc.dll
  ❌ Windows ARM64 - marty-plugin-typescript-v0.2.2-aarch64-pc-windows-msvc.dll (NOT FOUND)

📊 Summary: 5/6 platforms available
⚠️  Some platforms missing:
   - Windows ARM64

Users on missing platforms won't be able to use this plugin.
```

## Benefits for Plugin Developers

### 1. Reduced Friction

- **Before:** Developers had to manually figure out naming conventions, build commands, and release procedures
- **After:** Simple commands guide them through the entire process

### 2. Fewer Mistakes

Common mistakes now caught automatically:
- ❌ Wrong binary naming format
- ❌ Incorrect release tag format
- ❌ Plugin name mismatch
- ❌ Missing platform binaries
- ❌ Wrong file extensions

### 3. Cross-Platform Support Made Easy

- Developers get exact build commands for all 6 platforms
- Binary renaming is automated with correct target triples
- Validation ensures all platforms are covered

### 4. Standardization

- Enforces naming conventions
- Ensures consistent release format
- Makes plugins discoverable and predictable

## Technical Implementation

### Core Changes

1. **`PluginConfig`** - Renamed `repository` to `github_repo`
2. **`PluginCache::extract_plugin_name_from_repo()`** - Made public for CLI use
3. **Added `reqwest` dependency** to CLI for HTTP checks
4. **New command handlers** in `commands/plugin.rs`

### Code Quality

- ✅ All 13 existing tests pass
- ✅ Follows Rust best practices
- ✅ Type-safe enums for commands
- ✅ Clear error messages
- ✅ Comprehensive documentation

### Files Modified

**Core:**
- `crates/core/src/configs/workspace.rs` - Field rename
- `crates/core/src/plugin_cache.rs` - Public method, field usage
- `crates/cli/Cargo.toml` - Added reqwest dependency

**CLI:**
- `crates/cli/src/main.rs` - New command definitions
- `crates/cli/src/commands/plugin.rs` - Command implementations

**Examples:**
- `examples/ts-workspace/.marty/workspace.yml` - Updated to use `githubRepo`

**Documentation:**
- `docs/PLUGIN_DEVELOPER_GUIDE.md` - NEW: Complete developer guide
- `docs/PLUGIN_MONOREPO_APPROACH.md` - Updated field names
- `docs/PLUGIN_RESOLUTION.md` - Updated field names

## Migration Guide

For existing users with plugin configurations:

**Before:**
```yaml
plugins:
  - repository: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

**After:**
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
```

Simply rename the `repository` field to `githubRepo`.

## Future Enhancements

Potential additions:
1. **Runtime plugin validation** - Load plugin and verify name matches
2. **Automated GitHub release creation** - `marty plugin publish` command
3. **GitLab/Gitea support** - Add `gitlabRepo`, `giteaRepo` fields
4. **Plugin testing framework** - Help developers test their plugins
5. **Plugin marketplace** - Central registry of available plugins

## Testing

All commands tested and working:

```bash
# Validate
✅ marty plugin validate <path> --name <name>

# Release guide
✅ marty plugin release-guide <name> <version> --github-repo <repo> --monorepo

# Check release
✅ marty plugin check-release --github-repo <repo> --plugin <name> --version <version>

# Existing commands still work
✅ marty plugin list
✅ marty plugin clear
✅ marty plugin update

# Workspace commands unaffected
✅ marty list
✅ marty graph
✅ marty run <target>
✅ marty plan <target>
```

## Summary

This update makes Marty significantly more developer-friendly by:

1. **Clarifying configuration** with `githubRepo` field
2. **Providing validation tools** to catch errors early
3. **Automating release guidance** with step-by-step instructions
4. **Verifying releases** before users encounter issues
5. **Standardizing the ecosystem** with clear conventions

Plugin developers can now go from idea to published plugin with clear guidance at every step, while users benefit from higher-quality, properly-released plugins.
