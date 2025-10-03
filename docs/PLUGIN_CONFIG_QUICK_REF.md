# Plugin Configuration Quick Reference

## TL;DR

Use GitHub convention for cross-platform compatibility:

```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
```

## Three Ways to Configure Plugins

| Method | When to Use | Cross-Platform | Example |
|--------|-------------|----------------|---------|
| **GitHub Convention** | Production, shared configs | ✅ Yes | `repository: "owner/marty-plugin-name"` + `version: "0.2.0"` |
| **Direct URL** | Custom hosting, testing | ❌ No | `url: "https://example.com/plugin.so"` |
| **Local Path** | Development, internal tools | ❌ No | `path: "/path/to/plugin.so"` |

## Complete Examples

### GitHub Convention (Recommended)
```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    options:
      includes: ["crates/**"]
```

### Direct URL
```yaml
plugins:
  - url: "https://github.com/user/plugin/releases/download/v1.0.0/plugin-linux-x64.so"
    options: {}
```

### Local Path
```yaml
plugins:
  - path: "/absolute/path/to/plugin.so"
  - path: "builtin"  # Special: .marty/plugins/
```

## Supported Platforms

| OS | Architecture | Auto-Detected |
|----|--------------|---------------|
| Linux | x86_64 | ✅ |
| Linux | ARM64 | ✅ |
| macOS | x86_64 | ✅ |
| macOS | ARM64 (M1/M2) | ✅ |
| Windows | x86_64 | ✅ |
| Windows | ARM64 | ✅ |

## Repository Naming

✅ **Required format:** `marty-plugin-{name}`

```yaml
# ✅ Correct
- repository: "codyspate/marty-plugin-cargo"

# ❌ Wrong
- repository: "codyspate/cargo"
```

## Common Patterns

### Multiple Plugins
```yaml
plugins:
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
  - repository: "codyspate/marty-plugin-typescript"
    version: "0.2.1"
    options:
      auto_project_references: true
```

### Development Override
```yaml
plugins:
  # Production
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
    enabled: false  # Temporarily disabled
  
  # Development version
  - path: "../my-plugin/target/release/plugin.so"
    enabled: true
```

### Mixed Hosting
```yaml
plugins:
  # Public plugin via GitHub
  - repository: "codyspate/marty-plugin-cargo"
    version: "0.2.0"
  
  # Private plugin via company server
  - url: "https://internal.company.com/plugins/company-plugin.so"
```

## Troubleshooting

### Plugin Not Found (404)
```
Error: Failed to download plugin: HTTP 404 Not Found
```

**Fix:** Check version exists on GitHub releases:
```bash
curl -s https://api.github.com/repos/owner/marty-plugin-name/releases \
  | jq '.[].tag_name'
```

### Invalid Repository Name
```
Error: Repository name must start with 'marty-plugin-'
```

**Fix:** Use correct format:
```yaml
# Change this:
repository: "owner/cargo"

# To this:
repository: "owner/marty-plugin-cargo"
```

### Platform Not Supported
```
Error: Unsupported platform: freebsd-x86_64
```

**Fix:** Use local build or direct URL:
```yaml
- path: "/usr/local/plugins/plugin.so"
```

## Best Practices

1. ✅ **Pin versions**: Use exact versions, not ranges
2. ✅ **Document options**: Add comments for configuration
3. ✅ **Use GitHub convention**: Prefer repository + version
4. ✅ **Test offline**: Verify cached plugins work
5. ✅ **Disable, don't delete**: Use `enabled: false` to preserve config

## More Information

- [Full Documentation](../docs/PLUGIN_RESOLUTION.md)
- [Configuration Examples](../docs/PLUGIN_CONFIGURATION_EXAMPLES.md)
- [Plugin Development](../docs/PLUGIN_DEVELOPMENT.md)
