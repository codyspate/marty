# GitHub Convention Plugin Resolution - Implementation Summary

## Overview

Implemented a clean, developer-friendly plugin resolution system that automatically handles cross-platform plugin loading using GitHub releases and standard naming conventions.

## Problem Solved

**Challenge**: Users needed to specify platform-specific plugin URLs in workspace configuration, making configs non-portable across different operating systems and architectures.

**Solution**: GitHub convention with automatic platform detection - users specify just repository + version, Marty handles platform-specific URL generation.

## Implementation

### 1. Core Components

#### Platform Detection (`crates/core/src/platform.rs`)
```rust
pub struct PlatformInfo {
    pub target: &'static str,      // e.g., "x86_64-unknown-linux-gnu"
    pub extension: &'static str,   // e.g., "so", "dylib", "dll"
}
```

**Supported Platforms:**
- Linux: x86_64, aarch64 (`.so`)
- macOS: x86_64, aarch64/Apple Silicon (`.dylib`)
- Windows: x86_64, aarch64 (`.dll`)

#### Plugin Configuration (`crates/core/src/configs/workspace.rs`)
```rust
pub struct PluginConfig {
    pub repository: Option<String>,  // "owner/marty-plugin-name"
    pub version: Option<String>,     // "0.2.0"
    pub url: Option<String>,         // Direct URL fallback
    pub path: Option<String>,        // Local path fallback
    pub enabled: Option<bool>,
    pub options: Option<Value>,
}
```

**Resolution Priority:**
1. GitHub convention (`repository` + `version`)
2. Direct URL (`url`)
3. Local path (`path`)

#### Plugin Cache Updates (`crates/core/src/plugin_cache.rs`)

**Key Methods:**
- `resolve_github_plugin_url()` - Constructs GitHub release URL
- `extract_plugin_name_from_repo()` - Validates and extracts plugin name
- Platform-aware URL generation following naming convention

**URL Format:**
```
https://github.com/{owner}/{repo}/releases/download/v{version}/marty-plugin-{name}-v{version}-{target}.{ext}
```

### 2. Configuration Formats

#### GitHub Convention (Recommended)
```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      includes: ["crates/**"]
```

#### Direct URL (Fallback)
```yaml
plugins:
  - url: "https://custom-host.com/plugin.so"
    options: {}
```

#### Local Path (Development)
```yaml
plugins:
  - path: "/path/to/plugin.so"
    options: {}
```

### 3. Repository Naming Convention

**Required Format:** `marty-plugin-{name}`

**Examples:**
- ✅ `codyspate/marty-plugin-cargo`
- ✅ `user/marty-plugin-typescript`
- ❌ `user/cargo` (missing prefix)
- ❌ `user/cargo-plugin` (wrong prefix)

### 4. Plugin Binary Naming Convention

**Format:** `marty-plugin-{name}-v{version}-{target}.{ext}`

**Examples:**
```
marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so
marty-plugin-cargo-v0.2.0-aarch64-apple-darwin.dylib
marty-plugin-cargo-v0.2.0-x86_64-pc-windows-msvc.dll
```

## Benefits

### For Users

1. **Cross-Platform Support**
   - Same config works on all operating systems
   - No platform-specific branches needed
   - Team members on different OSes use identical config

2. **Simplicity**
   ```yaml
   # Before (platform-specific)
   - url: "https://github.com/.../plugin-v0.2.0-x86_64-unknown-linux-gnu.so"
   
   # After (cross-platform)
   - repository: "owner/marty-plugin-cargo"
     version: "0.2.0"
   ```

3. **Automatic Updates**
   - Just change version number
   - Marty handles URL construction
   - No manual URL updates needed

4. **Free Hosting**
   - Uses GitHub releases (no cost)
   - Leverages existing infrastructure
   - Familiar workflow for developers

### For Plugin Authors

1. **Standard Workflow**
   - Follow simple naming conventions
   - Use GitHub Actions for releases
   - Automatic platform detection

2. **Discoverability**
   - Clear naming pattern for search
   - Standard GitHub repository structure
   - Easy to find and install

3. **No Infrastructure Needed**
   - GitHub handles hosting and CDN
   - No custom servers required
   - Free for open source

## Testing

### Platform Detection Tests
```bash
cargo test --package marty_core platform
```

**Coverage:**
- Linux x86_64 and aarch64
- macOS x86_64 and aarch64
- Windows x86_64 and aarch64
- Unsupported platform detection

### Plugin Resolution Tests
```bash
cargo test --package marty_core plugin_cache
```

**Coverage:**
- Repository name extraction
- GitHub URL generation
- Invalid format handling
- Cross-platform URL validation

**Results:** ✅ All tests passing

## Documentation

### User Documentation
- [`PLUGIN_RESOLUTION.md`](./PLUGIN_RESOLUTION.md) - Comprehensive guide
- [`PLUGIN_CONFIGURATION_EXAMPLES.md`](./PLUGIN_CONFIGURATION_EXAMPLES.md) - Example configs
- Updated README with new format examples

### Key Topics Covered
- Configuration format options
- Platform support matrix
- Naming conventions
- Migration guides
- Troubleshooting
- Best practices

## Migration Path

### Phase 1: Current Implementation ✅
- GitHub convention support
- Backward compatible with URL/path
- Platform detection
- Automatic URL generation

### Phase 2: Plugin Ecosystem (Future)
- Plugin template repository
- GitHub Actions for automated releases
- Cross-compilation setup
- Plugin testing framework

### Phase 3: Enhanced Discovery (Future)
- Plugin registry (optional)
- Version range support (semver)
- Plugin search/discovery commands
- Automatic update notifications

## Usage Examples

### Real-World Configuration
```yaml
name: "Production Workspace"
description: "Multi-language monorepo"

plugins:
  # Rust workspaces
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      includes: ["crates/**", "plugins/**"]
      excludes: ["**/target/**"]
  
  # TypeScript projects
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    options:
      auto_project_references: true
      reference_path_style: "relative"
  
  # PNPM workspaces
  - repository: "codyspate/marty-plugin-pnpm"
    version: "0.2.0"
    options:
      includes: ["**/package.json"]
```

### Development Configuration
```yaml
plugins:
  # Production plugins
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
  
  # Local development plugin
  - path: "../my-plugin/target/release/libmy_plugin.so"
    enabled: true  # Override for testing
```

## Error Handling

### Comprehensive Error Messages

**Invalid Repository Format:**
```
Error: Repository name must start with 'marty-plugin-': 'cargo'
Example: 'owner/marty-plugin-cargo'
```

**Plugin Not Found:**
```
Error: Failed to download plugin from {url}: HTTP 404 Not Found

Possible causes:
- Version doesn't exist
- Binary not published for your platform (linux-x86_64)
- Repository or release deleted
```

**Platform Not Supported:**
```
Error: Unsupported platform: freebsd-x86_64

Supported platforms:
- linux-x86_64, linux-aarch64
- macos-x86_64, macos-aarch64
- windows-x86_64, windows-aarch64
```

## Performance

### Caching Strategy
- Plugins cached in `.marty/cache/plugins/`
- Cache key: plugin name + URL hash
- No re-download on cache hit
- Offline operation with cached plugins

### Network Efficiency
- HEAD request validation (future)
- Conditional downloads (future)
- Parallel plugin downloads (future)

## Future Enhancements

### 1. Plugin Registry
```yaml
plugins:
  - name: "cargo"
    version: "^0.2.0"  # Semantic versioning
```

### 2. CLI Commands
```bash
marty plugin search typescript
marty plugin info cargo
marty plugin update
marty plugin cache clear
```

### 3. Plugin Verification
- SHA256 checksum validation
- GPG signature verification
- Trusted plugin sources

### 4. Enhanced Resolution
- Version ranges (^0.2.0, ~1.0)
- Pre-release channels (beta, rc)
- Plugin aliases/shortcuts

## Architecture Decisions

### Why GitHub Convention?

1. **No Infrastructure**: Uses existing free GitHub services
2. **Familiar**: Developers already use GitHub for code
3. **Reliable**: GitHub's CDN and uptime
4. **Simple**: Just repository + version
5. **Flexible**: Falls back to URL when needed

### Why Not a Registry?

**Pros of Registry:**
- Centralized discovery
- Curated plugins
- Trust model

**Cons of Registry:**
- Infrastructure cost
- Maintenance burden
- Single point of failure
- Not necessary initially

**Decision:** Start with GitHub convention, consider registry later if needed.

### Why Three Resolution Methods?

1. **GitHub Convention**: Primary method for production
2. **Direct URL**: Fallback for custom hosting
3. **Local Path**: Development and testing

Provides flexibility while maintaining simplicity.

## Security Considerations

### Current
- HTTPS for all downloads
- Plugin validation on load
- Isolated plugin cache per workspace

### Future
- Checksum verification
- Code signing support
- Trusted plugin sources
- Audit logging

## Backward Compatibility

### All Existing Formats Supported

```yaml
# ✅ Old URL format still works
- url: "https://github.com/.../plugin.so"

# ✅ Old path format still works
- path: "/path/to/plugin.so"

# ✅ New GitHub convention
- repository: "owner/marty-plugin-cargo"
  version: "0.2.0"
```

### Migration Strategy
1. New projects use GitHub convention
2. Existing projects continue working
3. Gradual migration through documentation
4. Both formats supported indefinitely

## Success Criteria

✅ **Cross-platform support**: Same config works on all OSes
✅ **Simple configuration**: Just repository + version
✅ **Backward compatible**: Old configs still work
✅ **Well tested**: Comprehensive test coverage
✅ **Well documented**: Multiple docs and examples
✅ **Error handling**: Clear, actionable error messages
✅ **Performance**: Caching for offline operation

## Next Steps

### For Core Team
1. ✅ Implement GitHub convention resolution
2. ✅ Add platform detection
3. ✅ Write comprehensive tests
4. ✅ Create documentation
5. 🔄 Publish plugins to GitHub releases
6. 📋 Create plugin template repository
7. 📋 Build GitHub Actions for plugin CI/CD

### For Plugin Authors
1. 📋 Adopt naming convention (`marty-plugin-{name}`)
2. 📋 Use GitHub Actions for cross-platform builds
3. 📋 Follow binary naming convention
4. 📋 Publish to GitHub releases

### For Users
1. ✅ Use GitHub convention in new workspaces
2. 🔄 Migrate existing workspaces (optional)
3. 📋 Provide feedback on ergonomics
4. 📋 Report platform support issues

## Conclusion

The GitHub convention plugin resolution system provides a clean, simple, and cross-platform solution for plugin distribution. It leverages existing infrastructure (GitHub releases), follows familiar patterns, and maintains backward compatibility while significantly improving user experience.

**Key Achievement**: Users can now share workspace configurations across teams on different platforms without modification, making Marty truly cross-platform.
