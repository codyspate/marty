# Plugin Type System - Implementation Summary

## Overview

This document summarizes the implementation of explicit plugin types in Marty, making plugin roles and capabilities clear through type declarations.

## Changes Made

### 1. Plugin Protocol (`marty_plugin_protocol`)

#### New Types

**`PluginType` Enum** (`crates/plugin_protocol/src/lib.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// Plugin discovers projects and their workspace dependencies.
    Primary,
    /// Plugin enhances existing projects without discovering new ones.
    Supplemental,
    /// Plugin executes actions at lifecycle hooks without discovering projects.
    Hook,
}
```

**Helper Methods**:
- `discovers_projects() -> bool`: Returns true for Primary plugins
- `description() -> &'static str`: Human-readable description

#### Updated Trait

**`MartyPlugin` Trait**
- Added required method: `fn plugin_type(&self) -> PluginType`
- This is a **breaking change** - all plugins must implement this method

#### FFI Export

**New C Function** (in `export_plugin!` macro):
```rust
#[no_mangle]
pub extern "C" fn plugin_type() -> u8 {
    match PLUGIN.plugin_type() {
        PluginType::Primary => 0,
        PluginType::Supplemental => 1,
        PluginType::Hook => 2,
    }
}
```

### 2. Core (`marty_core`)

#### Dynamic Plugin Loader (`plugin_runtime_dylib.rs`)

**Changes**:
- Added `PluginTypeFn` FFI function type
- Added `plugin_type: PluginType` field to `DylibWorkspaceProvider`
- Added `get_plugin_type()` method to extract type from plugin library
- Implemented `plugin_type()` in `MartyPlugin` trait

**FFI Function Signature**:
```rust
type PluginTypeFn = unsafe extern "C" fn() -> u8;
```

#### Configurable Provider (`workspace_manager.rs`)

**Changes**:
- Implemented `plugin_type()` by delegating to inner plugin
- Maintains type transparency through wrapper

### 3. Plugins

#### TypeScript Plugin
**Status**: Updated to `PluginType::Supplemental`
```rust
fn plugin_type(&self) -> PluginType {
    PluginType::Supplemental
}
```
**Rationale**: Enhances projects discovered by PNPM/NPM, doesn't discover projects itself

#### PNPM Plugin
**Status**: Updated to `PluginType::Primary`
```rust
fn plugin_type(&self) -> PluginType {
    PluginType::Primary
}
```
**Rationale**: Discovers JavaScript/TypeScript projects from package.json files

#### Cargo Plugin
**Status**: Updated to `PluginType::Primary`
```rust
fn plugin_type(&self) -> PluginType {
    PluginType::Primary
}
```
**Rationale**: Discovers Rust projects from Cargo.toml files

## Plugin Type Definitions

### Primary Plugins
- **Purpose**: Discover projects and workspace dependencies
- **Examples**: PNPM, Cargo, NPM
- **Behavior**: 
  - Return non-empty `include_path_globs()`
  - Can return `Some(InferredProject)` from `on_file_found()`
  - Multiple Primary plugins can coexist

### Supplemental Plugins
- **Purpose**: Enhance existing projects without discovering new ones
- **Examples**: TypeScript (adds project references)
- **Behavior**:
  - Return empty `include_path_globs()` or `vec![]`
  - Always return `None` from `on_file_found()`
  - Process projects through separate mechanisms

### Hook Plugins
- **Purpose**: Execute actions at lifecycle points
- **Examples**: (future) Pre-commit hooks, linters
- **Status**: Planned but not yet implemented
- **Behavior**: 
  - Return empty `include_path_globs()`
  - Always return `None` from `on_file_found()`
  - Will implement lifecycle hook methods (to be defined)

## Breaking Changes

### For Plugin Developers

**Required**: All plugins must implement the new `plugin_type()` method:

```rust
impl MartyPlugin for MyPlugin {
    fn plugin_type(&self) -> PluginType {
        PluginType::Primary  // or Supplemental or Hook
    }
    
    // ... existing methods
}
```

**Import Update**: Add `PluginType` to imports:

```rust
use marty_plugin_protocol::{
    dylib::export_plugin,
    InferredProject,
    MartyPlugin,
    PluginType,  // ADD THIS
    Workspace,
    WorkspaceProvider,
};
```

### Migration Steps

1. Add `PluginType` to imports
2. Implement `plugin_type()` method
3. Choose appropriate type:
   - **Primary**: If plugin discovers projects
   - **Supplemental**: If plugin enhances existing projects
   - **Hook**: If plugin runs lifecycle actions (future)
4. Rebuild plugin
5. Test with Marty

## Benefits

### Type Safety
- Compile-time enforcement of plugin capabilities
- Prevents accidental misuse
- Self-documenting code

### Performance
- Marty can skip file scanning for non-Primary plugins
- Better optimization opportunities
- Reduced unnecessary I/O

### Developer Experience
- Clear plugin purpose at a glance
- Easier to understand plugin ecosystem
- Better error messages

### Validation
- Runtime checks ensure plugins behave correctly
- Warning system for violations
- Future: strict enforcement

## File Changes Summary

```
crates/plugin_protocol/src/lib.rs
├── Added PluginType enum with helper methods
├── Updated MartyPlugin trait with plugin_type() method
└── Updated export_plugin! macro to export plugin_type FFI function

crates/core/src/plugin_runtime_dylib.rs
├── Added PluginTypeFn type alias
├── Added plugin_type field to DylibWorkspaceProvider
├── Added get_plugin_type() method
└── Implemented plugin_type() in MartyPlugin impl

crates/core/src/workspace_manager.rs
└── Implemented plugin_type() in ConfigurableWorkspaceProvider

plugins/typescript/src/lib.rs
├── Added PluginType import
└── Implemented plugin_type() as Supplemental

plugins/pnpm/src/lib.rs
├── Added PluginType import
└── Implemented plugin_type() as Primary

plugins/cargo/src/lib.rs
├── Added PluginType import
└── Implemented plugin_type() as Primary

docs/PLUGIN_TYPES.md (NEW)
└── Comprehensive documentation of plugin type system

docs/PLUGIN_TYPE_IMPLEMENTATION_SUMMARY.md (NEW)
└── This file
```

## Testing

### Build Status
- ✅ `marty_plugin_protocol` builds successfully
- ✅ `marty_core` builds successfully
- ✅ `marty_cli` builds successfully
- ✅ All plugins build successfully

### Test Status
- ✅ TypeScript plugin tests pass (5/5)
- ✅ Plugin protocol compiles without errors
- ✅ No breaking changes in test suite

## Future Enhancements

### Short Term
1. Add validation warnings when Supplemental/Hook plugins return projects
2. Document plugin type in `marty list` command output
3. Add plugin type filter to CLI commands

### Medium Term
1. Implement lifecycle hooks for Hook-type plugins
2. Add plugin dependency ordering based on types
3. Performance optimizations for non-Primary plugins

### Long Term
1. Plugin composition and chaining
2. Dynamic plugin type switching (with safeguards)
3. Plugin capability negotiation

## Version Impact

**Plugin Protocol Version**: 0.3.0 → 0.4.0 (breaking change)
- New required trait method: `plugin_type()`
- All existing plugins must update

**Recommendation**: 
- Update all first-party plugins immediately
- Provide migration guide for third-party plugins
- Consider grace period with deprecation warnings

## References

- [Plugin Types Documentation](./PLUGIN_TYPES.md)
- [Plugin Developer Guide](./PLUGIN_DEVELOPER_GUIDE.md)
- [TypeScript Plugin Architecture](./TYPESCRIPT_PLUGIN_ARCHITECTURE.md)
- [Plugin Protocol Source](../crates/plugin_protocol/src/lib.rs)
