# TypeScript Workspace Example

This is an example TypeScript monorepo demonstrating:

- **PNPM Workspace**: Multi-package repository management
- **TypeScript Project References**: Optimized builds with proper dependency management
- **Marty Integration**: Workspace detection and task orchestration

## Structure

```
ts-workspace/
├── package-a/          # Shared utilities library
│   ├── src/
│   │   └── index.ts    # Exported utilities
│   ├── package.json    # Package configuration
│   └── tsconfig.json   # TypeScript configuration
├── package-b/          # Application using shared utilities
│   ├── src/
│   │   └── index.ts    # Application code
│   ├── package.json    # Package configuration
│   └── tsconfig.json   # TypeScript configuration with references
├── package.json        # Workspace root configuration
├── pnpm-workspace.yaml # PNPM workspace configuration
├── tsconfig.json       # Root TypeScript configuration
├── tsconfig.base.json  # Shared TypeScript configuration
└── .marty/
    └── workspace.yml   # Marty workspace configuration
```

## Features

### PNPM Workspace
- **Efficient dependency management** with shared node_modules
- **Workspace protocol** for internal package dependencies
- **Unified scripts** that run across all packages

### TypeScript Project References
- **Incremental builds** - only rebuild what's necessary
- **Proper build ordering** - dependencies built first
- **Composite projects** - optimized for monorepo usage
- **Auto project references** - automatically maintained by Marty TypeScript plugin

### Marty Integration
- **Automatic project detection** via TypeScript and PNPM plugins
- **Dependency graph analysis** for build ordering
- **Task orchestration** across workspace packages
- **Auto-updating project references** when dependencies change

## Getting Started

### 1. Install Dependencies

```bash
pnpm install
```

### 2. Build All Packages

```bash
pnpm build
```

This will build packages in the correct dependency order:
1. `package-a` (no dependencies)
2. `package-b` (depends on package-a)

### 3. Run the Application

```bash
cd package-b
node dist/index.js
```

### 4. Development Mode

```bash
# Watch mode for all packages
pnpm dev
```

## Available Scripts

### Workspace Level (run from root)

- `pnpm build` - Build all packages
- `pnpm dev` - Watch mode for all packages  
- `pnpm test` - Run tests for all packages
- `pnpm clean` - Clean build outputs for all packages
- `pnpm typecheck` - Type check all packages

### Package Level

Each package has its own scripts:

- `pnpm build` - Build this package
- `pnpm dev` - Watch mode for this package
- `pnpm clean` - Clean build output
- `pnpm typecheck` - Type check this package

## Marty Features

### Project Detection

Marty automatically detects projects and dependencies through a plugin-based architecture:

1. **TypeScript Projects**: TypeScript plugin detects projects via `tsconfig.json` files
2. **Workspace Dependencies**: Built-in package.json detection or PNPM plugin analyzes `package.json` files and `pnpm-workspace.yaml`
3. **Separation of Concerns**: Each plugin focuses on its specific domain - TypeScript for TS configuration, PNPM for workspace dependencies

### Auto Project References

When `auto_project_references: true` is enabled, the TypeScript plugin will:

1. **Receive workspace dependencies** detected by other plugins (PNPM plugin or built-in package.json detection)
2. **Update tsconfig.json** with proper project references based on those dependencies
3. **Maintain references** automatically when dependencies change

This creates a clean separation where the PNPM plugin handles workspace dependency detection and the TypeScript plugin handles TypeScript-specific configuration management.

Example: When package-b depends on package-a, its `tsconfig.json` automatically gets:

```json
{
  "references": [
    { "path": "../package-a" }
  ]
}
```

### Task Orchestration

Marty understands the dependency graph and can:

- **Run builds in order**: Dependencies before dependents
- **Parallel execution**: Independent packages built simultaneously  
- **Change detection**: Only rebuild affected packages
- **Task coordination**: Ensure prerequisites are met

## TypeScript Features

### Composite Projects

Each package uses `"composite": true` which enables:

- **Declaration files** (.d.ts) generation
- **Build info** caching for faster incremental builds
- **Project references** for cross-package type checking

### Incremental Builds

TypeScript's project references enable:

- **Fast rebuilds**: Only changed packages are rebuilt
- **Dependency tracking**: Changes propagate correctly
- **Editor support**: Go-to-definition across packages
- **Type safety**: Full type checking across project boundaries

## Best Practices Demonstrated

### Package Structure
- **Source in src/**: Clean separation of source and build output
- **Proper exports**: ESM exports with types
- **Composite setup**: Optimized for monorepo usage

### Dependency Management  
- **Workspace protocol**: Use `workspace:*` for internal dependencies
- **Proper scoping**: Use organization scope (@repo/) for internal packages
- **Clean boundaries**: Clear interface between packages

### Build Configuration
- **Shared base config**: Common TypeScript settings in `tsconfig.base.json`
- **Project-specific**: Each package extends base with specific needs
- **Incremental**: Build info and composite projects for speed

### Marty Integration
- **Plugin configuration**: TypeScript and PNPM plugins properly configured
- **Auto-maintenance**: Project references updated automatically
- **Workspace organization**: Clear includes/excludes for scanning

This example serves as a template for creating TypeScript monorepos with optimal tooling integration.

## Testing the Example

### 1. Install Dependencies
```bash
pnpm install
```

### 2. Build All Packages
```bash
pnpm build
```

### 3. Test Marty Integration

From the Marty repository root:

```bash
# List detected projects
./target/debug/marty --workspace examples/ts-workspace list

# Show project dependency graph  
./target/debug/marty --workspace examples/ts-workspace graph

# Build using Marty (respects dependency order)
./target/debug/marty --workspace examples/ts-workspace run build

# Clean build outputs
./target/debug/marty --workspace examples/ts-workspace run clean
```

### What Works ✅

- **PNPM Workspace**: Multi-package repository with workspace protocol dependencies
- **TypeScript Project References**: Automatic composite project setup with proper references
- **Marty Integration**: Project detection, dependency graph analysis, and task execution
- **Auto Project References**: TypeScript plugin automatically manages project references based on workspace dependencies
- **Build Order**: Marty respects dependency order when running tasks (package-a builds before package-b)
- **Task Management**: Individual project tasks defined in `marty.yml` files
- **Incremental Builds**: TypeScript composite projects enable fast incremental compilation
- **Cross-package Types**: Full type checking and editor support across project boundaries