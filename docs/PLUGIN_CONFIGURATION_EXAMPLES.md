# Example Workspace Configuration

This file demonstrates the different plugin configuration formats supported by Marty.

## GitHub Convention (Recommended)

Use this format for plugins hosted on GitHub with standard naming:

```yaml
name: "Example Workspace"
description: "Demonstrates Marty plugin configuration"

plugins:
  # Cargo plugin for Rust workspaces
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      includes: ["crates/**"]
      excludes: ["**/target/**"]
  
  # TypeScript plugin with auto project references
  - repository: "codyspate/marty-plugin-typescript"  
    version: "0.2.1"
    options:
      auto_project_references: true
      reference_path_style: "relative"
```

### Benefits
- ✅ Cross-platform: Works on Linux, macOS, and Windows automatically
- ✅ Simple: Just repository and version
- ✅ Shareable: Same config file works for entire team
- ✅ Free hosting: Uses GitHub releases

### How It Works
Marty automatically:
1. Detects your OS and architecture
2. Constructs the appropriate download URL
3. Downloads and caches the platform-specific binary

Example URL generation for Linux x86_64:
```
https://github.com/codyspate/marty-plugin-cargo/releases/download/v0.2.0/marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so
```

## Direct URL (Fallback)

Use when you need to specify exact plugin URL:

```yaml
plugins:
  # Custom hosted plugin
  - url: "https://custom-server.com/plugins/my-plugin-v1.0.0-linux-x64.so"
    options:
      custom_setting: true
      
  # Development/testing with specific binary
  - url: "https://github.com/user/repo/releases/download/v0.1.0-rc1/plugin-rc.so"
```

### When to Use
- Custom plugin hosting
- Non-standard binary naming
- Testing pre-release versions
- Private artifact servers

### Limitations
- ❌ Not cross-platform (different URLs per OS)
- ⚠️ Manual version management

## Local Path (Development)

Use for local development or pre-installed plugins:

```yaml
plugins:
  # Absolute path
  - path: "/home/user/projects/my-plugin/target/release/libmy_plugin.so"
    options:
      debug: true
      
  # Relative path (from workspace root)
  - path: ".marty/plugins/development-plugin.dylib"
    enabled: true
    
  # Built-in plugins (special handling)
  - path: "builtin"
    enabled: false
```

### When to Use
- Plugin development and testing
- CI/CD with pre-installed plugins
- Offline/airgapped environments
- Internal company plugins

## Mixed Configuration

You can combine different formats as needed:

```yaml
plugins:
  # Production plugins via GitHub
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    
  # Custom internal plugin via URL  
  - url: "https://internal.company.com/plugins/company-plugin.so"
    options:
      api_key: "${COMPANY_API_KEY}"
      
  # Development plugin for testing
  - path: "./local-plugins/test-plugin.so"
    enabled: false  # Disabled by default
```

## Platform Support

Marty automatically handles these platforms:

| OS | Architecture | Target | Extension |
|----|--------------|--------|-----------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | `.so` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `.so` |
| macOS | x86_64 | `x86_64-apple-darwin` | `.dylib` |
| macOS | ARM64 (M1/M2) | `aarch64-apple-darwin` | `.dylib` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `.dll` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `.dll` |

## Best Practices

### Version Pinning
Always pin to specific versions for reproducibility:

```yaml
# ✅ Good - exact version
- repository: "codyspate/marty-plugin-cargo"
  version: "0.2.0"

# ❌ Avoid - no version (will fail)
- repository: "codyspate/marty-plugin-cargo"
```

### Option Documentation
Document plugin options with comments:

```yaml
plugins:
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    options:
      # Automatically update tsconfig.json project references
      auto_project_references: true
      
      # Use relative paths (vs full path to tsconfig.json)
      reference_path_style: "relative"
```

### Disable Unused Plugins
Use `enabled: false` instead of deleting configuration:

```yaml
plugins:
  # Temporarily disabled for debugging
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    enabled: false  # Quick toggle without losing configuration
```

## Migration Examples

### From Old URL Format

**Before:**
```yaml
plugins:
  - url: "https://github.com/codyspate/marty/releases/download/marty-plugin-cargo-v0.2.0/marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so"
```

**After:**
```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
```

### From Local Development to Production

**Development:**
```yaml
plugins:
  - path: "../marty-plugins/cargo/target/release/libmarty_plugin_cargo.so"
    options:
      debug: true
```

**Production:**
```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    # Production options
```

## Troubleshooting

### Plugin Not Found (404)

If you get a 404 error:

1. Verify the version exists:
   ```bash
   # Check available releases
   curl -s https://api.github.com/repos/codyspate/marty-plugin-cargo/releases | jq '.[].tag_name'
   ```

2. Check if your platform is supported:
   ```bash
   # View release assets
   curl -s https://api.github.com/repos/codyspate/marty-plugin-cargo/releases/tags/v0.2.0 | jq '.assets[].name'
   ```

3. Fallback to direct URL temporarily:
   ```yaml
   - url: "https://github.com/codyspate/marty-plugin-cargo/releases/download/v0.2.0/marty-plugin-cargo-v0.2.0-x86_64-unknown-linux-gnu.so"
   ```

### Invalid Repository Name

```
Error: Repository name must start with 'marty-plugin-'
```

Repository must be named `marty-plugin-{name}`:
```yaml
# ❌ Wrong
- repository: "codyspate/cargo"

# ✅ Correct  
- repository: "codyspate/marty-plugin-cargo"
```

### Unsupported Platform

```
Error: Unsupported platform: freebsd-x86_64
```

Options:
1. Request platform support from plugin author
2. Build plugin locally for your platform
3. Use local path configuration

## See Also

- [Plugin Resolution Guide](../docs/PLUGIN_RESOLUTION.md) - Detailed resolution documentation
- [Plugin Development Guide](../docs/PLUGIN_DEVELOPMENT.md) - Creating your own plugins
- [GitHub Actions for Plugins](../docs/PLUGIN_CI_CD.md) - Automated plugin publishing
