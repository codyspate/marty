# TypeScript Plugin Auto Project References

This feature automatically adds project references to `tsconfig.json` files based on workspace dependencies detected by the TypeScript plugin.

## Configuration

Add the following configuration to your Marty workspace configuration:

```yaml
plugins:
  - name: typescript
    url: https://github.com/codyspate/marty/releases/download/marty-plugin-typescript-v0.2.1/marty-plugin-typescript-v0.2.1-x86_64-unknown-linux-gnu.so
    options:
      auto_project_references: true
      reference_path_style: "relative"  # or "tsconfig"
```

## Configuration Options

### `auto_project_references` (boolean, default: false)
When enabled, the plugin will automatically add TypeScript project references to `tsconfig.json` files based on detected workspace dependencies.

### `reference_path_style` (string, default: "relative")
Controls how project reference paths are generated:
- `"relative"`: Points to the dependency directory (e.g., `"../shared"`)
- `"tsconfig"`: Points directly to the tsconfig.json file (e.g., `"../shared/tsconfig.json"`)

## How It Works

1. **Detection**: The plugin scans for `tsconfig.json` files and analyzes project references to determine workspace dependencies
2. **Analysis**: When a TypeScript project imports from or depends on another workspace project, that dependency is recorded
3. **Updates**: If auto project references is enabled, the plugin updates `tsconfig.json` files to include proper project references

## Example

### Before Auto-Update

**Workspace Structure:**
```
my-workspace/
├── packages/
│   ├── shared/
│   │   ├── tsconfig.json
│   │   └── src/
│   ├── api/
│   │   ├── tsconfig.json  # Missing project references
│   │   └── src/
│   │       └── index.ts   # imports from '../shared'
│   └── ui/
│       ├── tsconfig.json  # Missing project references  
│       └── src/
│           └── app.tsx    # imports from '../shared'
```

**api/tsconfig.json (before):**
```json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist"
  }
}
```

### After Auto-Update

**api/tsconfig.json (after):**
```json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist"
  },
  "references": [
    { "path": "../shared" }
  ]
}
```

**ui/tsconfig.json (after):**
```json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist"
  },
  "references": [
    { "path": "../shared" }
  ]
}
```

## Benefits

1. **Build Performance**: TypeScript can build dependencies in the correct order
2. **Incremental Builds**: Only rebuilds what's necessary when dependencies change
3. **Type Safety**: Better type checking across project boundaries
4. **Editor Support**: Improved IDE navigation and refactoring across projects
5. **Maintenance**: No need to manually maintain project references

## Manual Usage

You can also use the standalone binary to update project references:

```bash
# Build the plugin with the binary
cargo build --release --bin update_references

# Update all TypeScript projects in the workspace
./target/release/update_references --workspace-root /path/to/workspace --auto-project-references

# Use tsconfig.json style references
./target/release/update_references --auto-project-references --reference-path-style tsconfig

# Dry run to see what would be updated
./target/release/update_references --auto-project-references --dry-run
```

## Integration with Marty Commands

The auto-update functionality can be triggered:

1. **Automatically**: When Marty discovers TypeScript projects (if enabled in config)
2. **On Command**: Using a custom Marty task or plugin command
3. **Manually**: Using the standalone binary

## Best Practices

1. **Enable Composite Mode**: Ensure your TypeScript projects have `"composite": true` in their `compilerOptions`
2. **Use Consistent Naming**: Project names should match directory names for best results
3. **Test Builds**: Verify that TypeScript builds work correctly after enabling auto references
4. **Version Control**: Commit the updated `tsconfig.json` files to ensure consistent builds across environments

## Troubleshooting

### References Not Being Added
- Ensure `auto_project_references: true` is set in plugin configuration
- Verify that workspace dependencies are being detected correctly
- Check that target projects have `tsconfig.json` files

### Build Errors After Update
- Ensure all referenced projects have `"composite": true` in their `compilerOptions`
- Verify that project names and paths are correct
- Check for circular dependencies between projects

### Path Resolution Issues
- Try switching between `"relative"` and `"tsconfig"` reference path styles
- Ensure all project directories are within the workspace root
- Verify that relative paths are calculated correctly