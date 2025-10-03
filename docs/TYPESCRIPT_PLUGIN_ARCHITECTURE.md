# TypeScript Plugin Architecture

## Overview

The TypeScript plugin has been redesigned as a **supplementary plugin** that enhances projects discovered by other plugins rather than discovering projects itself.

## Architecture Decision

### Problem

When both TypeScript and PNPM plugins were configured, they would both discover the same physical projects, leading to duplicates:
- TypeScript plugin found: `package-a`, `package-b` (from directory names)
- PNPM plugin found: `@repo/package-a`, `@repo/package-b` (from package.json names)

This resulted in:
```
Projects
@repo/package-a
@repo/package-b
package-a
package-b
```

### Solution

**Make TypeScript a supplementary plugin** that doesn't discover projects but provides TypeScript-specific functionality to projects discovered by package managers (PNPM, NPM, Yarn, etc.).

## How It Works

### Plugin Responsibility Split

| Plugin | Discovers Projects | Detects Dependencies | TypeScript Features |
|--------|-------------------|---------------------|---------------------|
| **PNPM/NPM** | ✅ Yes | ✅ Yes | ❌ No |
| **TypeScript** | ❌ No | ❌ No | ✅ Yes |

### TypeScript Plugin Capabilities

The TypeScript plugin provides **enhancement features** for discovered projects:

1. **Project Reference Management**
   - Automatically updates `tsconfig.json` with TypeScript project references
   - Based on workspace dependencies detected by other plugins
   - Configurable reference path style

2. **Configuration Options**
   ```yaml
   plugins:
     - githubRepo: codyspate/marty
       plugin: typescript
       version: 0.2.2
       options:
         auto_project_references: true
         reference_path_style: "relative"  # or "tsconfig"
   ```

### Project Discovery Flow

```
1. PNPM Plugin scans for package.json files
   ↓
2. PNPM Plugin discovers projects with:
   - Name from package.json (e.g., "@repo/package-a")
   - Workspace dependencies from package.json
   ↓
3. TypeScript Plugin (if enabled):
   - Finds projects with tsconfig.json
   - Updates project references based on workspace dependencies
   - Does NOT create new projects
```

## Benefits

### 1. No Duplicates
Only one plugin (PNPM/NPM) discovers each project, using the canonical name from `package.json`.

### 2. Proper Separation of Concerns
- **Package managers** handle project discovery and dependency detection
- **TypeScript plugin** handles TypeScript-specific enhancements

### 3. Flexible Configuration
Users can:
- Use PNPM plugin alone for basic project discovery
- Add TypeScript plugin for automatic project reference management
- Configure different combinations based on their needs

### 4. Accurate Project Names
Projects use their canonical names from `package.json` (including scopes):
```
@repo/package-a  ✅ (from package.json)
package-a        ❌ (would be from directory name)
```

## Configuration Examples

### Basic Setup (PNPM only)
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: pnpm
    version: 0.2.1
```

**Result:**
- Projects discovered from package.json
- Dependencies detected
- No TypeScript-specific features

### With TypeScript Enhancements
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: pnpm
    version: 0.2.1
    
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
    options:
      auto_project_references: true
      reference_path_style: "relative"
```

**Result:**
- Projects discovered from package.json
- Dependencies detected
- TypeScript project references automatically managed

## Implementation Details

### TypeScript Plugin Code

```rust
impl WorkspaceProvider for TypeScriptWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // Empty - doesn't discover projects
        vec![]
    }

    fn on_file_found(&self, _workspace: &Workspace, _path: &Path) -> Option<InferredProject> {
        // Always returns None - doesn't create projects
        None
    }
}
```

### Project Reference Updates

The TypeScript plugin processes ALL discovered projects (regardless of which plugin found them):

```rust
pub fn update_workspace_project_references(
    workspace: &Workspace,
    config_options: Option<&JsonValue>,
) -> anyhow::Result<Vec<String>> {
    // Process ALL inferred projects that have tsconfig.json
    for project in &workspace.inferred_projects {
        let tsconfig_path = project.project_dir.join("tsconfig.json");
        
        if tsconfig_path.exists() && !project.workspace_dependencies.is_empty() {
            // Update project references based on workspace dependencies
            update_project_references(
                &tsconfig_path,
                &project.workspace_dependencies,
                workspace,
                &config.reference_path_style,
            )?;
        }
    }
}
```

## Migration from Old Architecture

### Old Behavior (Pre-Refactor)
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.1  # Old version
```

**Result:**
- TypeScript plugin discovered projects from `tsconfig.json`
- Used directory names as project names
- Didn't detect workspace dependencies
- Resulted in duplicates when combined with PNPM plugin

### New Behavior (Current)
```yaml
plugins:
  # MUST include a package manager plugin for project discovery
  - githubRepo: codyspate/marty
    plugin: pnpm
    version: 0.2.1
    
  # TypeScript plugin is now optional and supplementary
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2
    options:
      auto_project_references: true
```

**Result:**
- PNPM plugin discovers projects from `package.json`
- Uses proper package names (with scopes)
- Detects workspace dependencies
- TypeScript plugin enhances with project references
- No duplicates

## Future Enhancements

Possible future features for the TypeScript plugin:

1. **Type Checking Integration**
   - Run `tsc` for type checking
   - Report errors in Marty's output

2. **Build Coordination**
   - Ensure TypeScript builds happen in dependency order
   - Share build artifacts across projects

3. **Configuration Validation**
   - Validate `tsconfig.json` settings across workspace
   - Ensure consistent compiler options

4. **Module Resolution**
   - Help configure TypeScript's `paths` for workspace dependencies
   - Auto-generate path mappings

## Best Practices

### DO ✅

- Configure a package manager plugin (PNPM, NPM, Yarn) as the primary project discoverer
- Use TypeScript plugin for TypeScript-specific enhancements
- Enable `auto_project_references` to keep references in sync
- Use scoped package names in package.json for clarity

### DON'T ❌

- Don't rely on TypeScript plugin alone for project discovery
- Don't expect TypeScript plugin to detect workspace dependencies
- Don't mix directory-based and package.json-based naming

## Troubleshooting

### Projects Not Appearing

**Problem:** No projects showing up in `marty list`

**Solution:** Ensure you have a package manager plugin configured:
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: pnpm
    version: 0.2.1
```

### Duplicate Projects

**Problem:** Seeing duplicate projects with different names

**Cause:** Likely using an old version of TypeScript plugin that still discovers projects

**Solution:** Update to latest version (0.2.2+):
```yaml
plugins:
  - githubRepo: codyspate/marty
    plugin: typescript
    version: 0.2.2  # Must be 0.2.2 or later
```

### Project References Not Updating

**Problem:** TypeScript project references not being generated

**Checklist:**
1. ✅ TypeScript plugin is configured
2. ✅ `auto_project_references: true` is set
3. ✅ Projects have `tsconfig.json` files
4. ✅ Projects have workspace dependencies detected by PNPM/NPM
5. ✅ Marty has write permissions to tsconfig.json files

## See Also

- [Plugin Resolution Guide](./PLUGIN_RESOLUTION.md)
- [Plugin Monorepo Approach](./PLUGIN_MONOREPO_APPROACH.md)
- [PNPM Plugin Documentation](../plugins/pnpm/README.md)
- [TypeScript Plugin Source](../plugins/typescript/src/lib.rs)
