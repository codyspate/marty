//! Dynamic library plugin interface for C ABI exports.
//!
//! This module provides the infrastructure for creating C ABI compatible plugins
//! that can be loaded as dynamic libraries (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows).
//!
//! ## Why Dynamic Libraries?
//!
//! Dynamic libraries provide better performance, easier debugging, and simpler deployment
//! compared to WASM while maintaining cross-platform compatibility.
//!
//! ## Usage
//!
//! Use the [`export_plugin!`] macro to export your plugin with all necessary C ABI functions.

/// Macro to export your plugin with a C ABI interface for dynamic library loading.
///
/// **Purpose**: This macro generates the necessary C-compatible functions that Marty
/// needs to interact with your plugin. It creates a bridge between Rust's type system
/// and the C ABI required for cross-language dynamic loading.
///
/// **Requirements**: Your plugin type must:
/// - Implement the `MartyPlugin` trait  
/// - Have a `const fn new() -> Self` constructor
/// - Be `Send + Sync` (automatically satisfied for most structs)
///
/// **Generated Functions**: The macro creates these C ABI exports:
/// - `plugin_name()` - Returns the plugin's display name
/// - `plugin_key()` - Returns the plugin's unique identifier
/// - `plugin_type()` - Returns the plugin type (Primary/Supplemental/Hook)
/// - `plugin_include_globs()` - Returns file inclusion patterns
/// - `plugin_exclude_globs()` - Returns file exclusion patterns
/// - `plugin_on_file_found()` - Handles file discovery events
/// - `plugin_cleanup_string()` - Manages memory for returned strings
/// - `plugin_config_options()` - Returns JSON schema configuration
///
/// # Usage
///
/// ```rust
/// use marty_plugin_protocol::{
///     dylib::export_plugin, InferredProject, MartyPlugin, PluginType,
///     Workspace, WorkspaceProvider,
/// };
/// use serde_json::{json, Value as JsonValue};
/// use std::path::Path;
///
/// // 1. Define your plugin struct
/// pub struct MyPlugin;
///
/// impl MyPlugin {
///     pub const fn new() -> Self {
///         Self
///     }
/// }
///
/// impl Default for MyPlugin {
///     fn default() -> Self {
///         Self::new()
///     }
/// }
///
/// // 2. Define your workspace provider  
/// pub struct MyWorkspaceProvider;
///
/// impl WorkspaceProvider for MyWorkspaceProvider {
///     fn include_path_globs(&self) -> Vec<String> {
///         vec!["**/my-config.json".to_string()]
///     }
///
///     fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
///         // Your detection logic here
///         None
///     }
/// }
///
/// // 3. Implement the MartyPlugin trait
/// impl MartyPlugin for MyPlugin {
///     fn plugin_type(&self) -> PluginType {
///         PluginType::Primary
///     }
///
///     fn name(&self) -> &str {
///         "My Framework Plugin"
///     }
///
///     fn key(&self) -> &str {
///         "my-framework"
///     }
///
///     fn workspace_provider(&self) -> &dyn WorkspaceProvider {
///         &MyWorkspaceProvider
///     }
///
///     fn configuration_options(&self) -> Option<JsonValue> {
///         Some(json!({
///             "type": "object",
///             "properties": {
///                 "version": {
///                     "type": "string",
///                     "description": "Framework version",
///                     "default": "latest"
///                 }
///             },
///             "additionalProperties": false
///         }))
///     }
/// }
///
/// // 4. Export the plugin - this MUST be the last line in your lib.rs
/// export_plugin!(MyPlugin);
/// ```
///
/// # Memory Management
///
/// The macro handles C string memory management automatically:
/// - Strings returned to Marty are allocated with `CString::into_raw()`
/// - Marty calls `plugin_cleanup_string()` to free the memory safely
/// - Plugins should never manually free strings returned to Marty
///
/// # Error Handling  
///
/// The generated C functions handle errors gracefully:
/// - Invalid UTF-8 strings return null pointers
/// - Parse errors in `on_file_found()` return null (no project detected)
/// - Memory allocation failures return null pointers
/// - Marty treats null returns as "no result" rather than errors
///
/// # Build Configuration
///
/// Your `Cargo.toml` must specify `cdylib` as the crate type:
///
/// ```toml
/// [lib]
/// crate-type = ["cdylib"]
/// ```
///
/// # Platform Compatibility
///
/// The generated dynamic library works on:
/// - **Linux**: `.so` files (e.g., `libmarty_plugin_cargo.so`)
/// - **macOS**: `.dylib` files (e.g., `libmarty_plugin_cargo.dylib`)  
/// - **Windows**: `.dll` files (e.g., `marty_plugin_cargo.dll`)
///
/// Marty automatically handles the platform-specific loading and naming conventions.
///
/// # Testing Your Plugin
///
/// After building with `cargo build --lib --release`, you can test the plugin by:
/// 1. Placing the dynamic library in Marty's plugin directory
/// 2. Running Marty in a workspace with your target project type
/// 3. Checking that your projects are detected correctly
///
/// **Plugin Directory Locations**:
/// - Linux/macOS: `~/.marty/plugins/`
/// - Windows: `%APPDATA%\marty\plugins\`
///
/// # Common Issues
///
/// **"Plugin not found"**: Ensure your Cargo.toml has `crate-type = ["cdylib"]`
///
/// **"Symbol not found"**: Verify `export_plugin!(YourPluginType)` is called exactly once
///
/// **"Memory errors"**: Never manually free strings returned to Marty; the macro handles this
///
/// **"Projects not detected"**: Check your `include_path_globs()` patterns match your target files
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        use std::ffi::{CStr, CString};
        use std::os::raw::c_char;

        static PLUGIN: $plugin_type = <$plugin_type>::new();

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const c_char {
            let name = PLUGIN.name();
            match CString::new(name) {
                Ok(cstr) => cstr.into_raw(),
                Err(_) => std::ptr::null(),
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_key() -> *const c_char {
            let key = PLUGIN.key();
            match CString::new(key) {
                Ok(cstr) => cstr.into_raw(),
                Err(_) => std::ptr::null(),
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_type() -> u8 {
            match PLUGIN.plugin_type() {
                $crate::PluginType::Primary => 0,
                $crate::PluginType::Supplemental => 1,
                $crate::PluginType::Hook => 2,
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_include_globs() -> *const c_char {
            let globs = PLUGIN.workspace_provider().include_path_globs();
            match serde_json::to_string(&globs) {
                Ok(json) => match CString::new(json) {
                    Ok(cstr) => cstr.into_raw(),
                    Err(_) => std::ptr::null(),
                },
                Err(_) => std::ptr::null(),
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_exclude_globs() -> *const c_char {
            let globs = PLUGIN.workspace_provider().exclude_path_globs();
            match serde_json::to_string(&globs) {
                Ok(json) => match CString::new(json) {
                    Ok(cstr) => cstr.into_raw(),
                    Err(_) => std::ptr::null(),
                },
                Err(_) => std::ptr::null(),
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_config_options() -> *const c_char {
            match PLUGIN.configuration_options() {
                Some(options) => match serde_json::to_string(&options) {
                    Ok(json) => match CString::new(json) {
                        Ok(cstr) => cstr.into_raw(),
                        Err(_) => std::ptr::null(),
                    },
                    Err(_) => std::ptr::null(),
                },
                None => std::ptr::null(),
            }
        }

        /// Safe wrapper for handling plugin file found logic
        fn handle_file_found_safe(
            path_ptr: *const c_char,
            _contents_ptr: *const c_char,
        ) -> Option<*const c_char> {
            // Validate pointers before any unsafe operations
            if path_ptr.is_null() {
                return None;
            }

            // Extract path string safely
            let path_str = unsafe {
                let path_cstr = CStr::from_ptr(path_ptr);
                match path_cstr.to_str() {
                    Ok(s) => s,
                    Err(_) => return None,
                }
            };

            let path = std::path::Path::new(path_str);

            // Create a minimal workspace context for the plugin
            let workspace = $crate::Workspace {
                root: std::path::PathBuf::from("."),
                projects: Vec::new(),
                inferred_projects: Vec::new(),
            };

            match PLUGIN.workspace_provider().on_file_found(&workspace, path) {
                Some(project) => {
                    let message = $crate::InferredProjectMessage::from(project);
                    match serde_json::to_string(&message) {
                        Ok(json) => match CString::new(json) {
                            Ok(cstr) => Some(cstr.into_raw()),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                }
                None => match CString::new("null") {
                    Ok(cstr) => Some(cstr.into_raw()),
                    Err(_) => None,
                },
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_on_file_found(
            path_ptr: *const c_char,
            contents_ptr: *const c_char,
        ) -> *const c_char {
            handle_file_found_safe(path_ptr, contents_ptr).unwrap_or_else(std::ptr::null)
        }

        /// Safe wrapper for cleaning up plugin-allocated strings
        fn cleanup_string_safe(ptr: *const c_char) {
            if !ptr.is_null() {
                unsafe {
                    let _ = CString::from_raw(ptr as *mut c_char);
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_cleanup_string(ptr: *const c_char) {
            cleanup_string_safe(ptr);
        }
    };
}

pub use export_plugin;
