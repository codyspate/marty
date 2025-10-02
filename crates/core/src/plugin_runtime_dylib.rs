use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use marty_plugin_protocol::{InferredProject, MartyPlugin, Workspace, WorkspaceProvider};
use serde_json::Value;

/// Plugin function signatures for the C ABI interface
type PluginNameFn = unsafe extern "C" fn() -> *const c_char;
type PluginKeyFn = unsafe extern "C" fn() -> *const c_char;
type PluginConfigOptionsFn = unsafe extern "C" fn() -> *const c_char;
type PluginOnFileFoundFn = extern "C" fn(*const c_char, *const c_char) -> *const c_char;
type PluginCleanupStringFn = extern "C" fn(*const c_char);

/// A workspace provider that loads and interacts with dynamic library plugins
pub struct DylibWorkspaceProvider {
    name: String,
    key: String,
    library: Library,
    _temp_dir: Option<tempfile::TempDir>, // Hold onto temp dir to prevent cleanup
    call_lock: Mutex<()>,                 // Prevent concurrent calls to the same plugin
}

impl DylibWorkspaceProvider {
    /// Get the name of this plugin
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the key of this plugin
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Load a plugin from a dynamic library file
    pub fn from_dylib(dylib_path: PathBuf) -> Result<Self> {
        // Load the dynamic library
        let library = unsafe {
            Library::new(&dylib_path).with_context(|| {
                format!("Failed to load plugin library: {}", dylib_path.display())
            })?
        };

        // Get plugin metadata
        let name = Self::get_plugin_name(&library)?;
        let key = Self::get_plugin_key(&library)?;

        Ok(Self {
            name,
            key,
            library,
            _temp_dir: None,
            call_lock: Mutex::new(()),
        })
    }

    /// Load a plugin from a dynamic library, creating a temporary copy if needed
    /// This is useful when loading from cache directories where the file might be locked
    pub fn from_dylib_with_temp_copy(dylib_path: PathBuf) -> Result<Self> {
        // Create a temporary directory and copy the library there
        let temp_dir =
            tempfile::tempdir().context("Failed to create temporary directory for plugin")?;

        let filename = dylib_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid plugin path"))?;

        let temp_path = temp_dir.path().join(filename);
        std::fs::copy(&dylib_path, &temp_path)
            .with_context(|| "Failed to copy plugin to temporary location".to_string())?;

        // Load from the temporary location
        let library = unsafe {
            Library::new(&temp_path).with_context(|| {
                format!("Failed to load plugin library: {}", temp_path.display())
            })?
        };

        // Get plugin metadata
        let name = Self::get_plugin_name(&library)?;
        let key = Self::get_plugin_key(&library)?;

        Ok(Self {
            name,
            key,
            library,
            _temp_dir: Some(temp_dir), // Keep temp dir alive
            call_lock: Mutex::new(()),
        })
    }

    /// Extract plugin name from the library
    fn get_plugin_name(library: &Library) -> Result<String> {
        unsafe {
            let name_fn: Symbol<PluginNameFn> = library
                .get(b"plugin_name")
                .context("Plugin missing plugin_name function")?;

            let name_ptr = name_fn();
            if name_ptr.is_null() {
                return Err(anyhow::anyhow!("Plugin name function returned null"));
            }

            let name_cstr = CStr::from_ptr(name_ptr);
            let name = name_cstr
                .to_str()
                .context("Plugin name contains invalid UTF-8")?
                .to_string();

            // Clean up the string if cleanup function exists
            if let Ok(cleanup_fn) =
                library.get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
            {
                cleanup_fn(name_ptr);
            }

            Ok(name)
        }
    }

    /// Extract plugin key from the library
    fn get_plugin_key(library: &Library) -> Result<String> {
        unsafe {
            let key_fn: Symbol<PluginKeyFn> = library
                .get(b"plugin_key")
                .context("Plugin missing plugin_key function")?;

            let key_ptr = key_fn();
            if key_ptr.is_null() {
                return Err(anyhow::anyhow!("Plugin key function returned null"));
            }

            let key_cstr = CStr::from_ptr(key_ptr);
            let key = key_cstr
                .to_str()
                .context("Plugin key contains invalid UTF-8")?
                .to_string();

            // Clean up the string if cleanup function exists
            if let Ok(cleanup_fn) =
                library.get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
            {
                cleanup_fn(key_ptr);
            }

            Ok(key)
        }
    }

    /// Call a plugin function that returns a JSON string
    fn call_json_function(&self, function_name: &[u8]) -> Result<Option<Value>> {
        let _guard = self.call_lock.lock().expect("plugin call mutex poisoned");

        unsafe {
            let func: Symbol<PluginConfigOptionsFn> = match self.library.get(function_name) {
                Ok(f) => f,
                Err(_) => return Ok(None), // Function not found, return None
            };

            let result_ptr = func();
            if result_ptr.is_null() {
                return Ok(None);
            }

            let result_cstr = CStr::from_ptr(result_ptr);
            let result_str = result_cstr
                .to_str()
                .context("Plugin function returned invalid UTF-8")?;

            if result_str.is_empty() {
                // Clean up and return None for empty results
                if let Ok(cleanup_fn) = self
                    .library
                    .get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
                {
                    cleanup_fn(result_ptr);
                }
                return Ok(None);
            }

            let value = serde_json::from_str(result_str).with_context(|| {
                format!("Plugin function returned invalid JSON: {}", result_str)
            })?;

            // Clean up the string if cleanup function exists
            if let Ok(cleanup_fn) = self
                .library
                .get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
            {
                cleanup_fn(result_ptr);
            }

            Ok(Some(value))
        }
    }
}

impl WorkspaceProvider for DylibWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        match self.call_json_function(b"plugin_include_globs") {
            Ok(Some(Value::Array(arr))) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => Vec::new(), // Return empty to use defaults
        }
    }

    fn exclude_path_globs(&self) -> Vec<String> {
        match self.call_json_function(b"plugin_exclude_globs") {
            Ok(Some(Value::Array(arr))) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => Vec::new(), // Return empty to use defaults
        }
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        let _guard = self.call_lock.lock().expect("plugin call mutex poisoned");

        let contents = std::fs::read_to_string(path).ok()?;
        let path_str = path.to_string_lossy();

        let path_cstr = CString::new(path_str.as_ref()).ok()?;
        let contents_cstr = CString::new(contents).ok()?;

        let func: Symbol<PluginOnFileFoundFn> = unsafe {
            self.library.get(b"plugin_on_file_found").ok()?
        };

        let result_ptr = func(path_cstr.as_ptr(), contents_cstr.as_ptr());
        if result_ptr.is_null() {
            return None;
        }

        let result_str = unsafe {
            let result_cstr = CStr::from_ptr(result_ptr);
            result_cstr.to_str().ok()?
        };

        if result_str.trim().is_empty() || result_str == "null" {
            // Clean up and return None
            if let Ok(cleanup_fn) = unsafe {
                self
                    .library
                    .get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
            } {
                cleanup_fn(result_ptr);
            }
            return None;
        }

        let value: Value = serde_json::from_str(result_str).ok()?;
        if value.is_null() {
            // Clean up and return None
            if let Ok(cleanup_fn) = unsafe {
                self
                    .library
                    .get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
            } {
                cleanup_fn(result_ptr);
            }
            return None;
        }

        let message: marty_plugin_protocol::InferredProjectMessage =
            serde_json::from_value(value).ok()?;

        // Clean up the string if cleanup function exists
        if let Ok(cleanup_fn) = unsafe {
            self
                .library
                .get::<Symbol<PluginCleanupStringFn>>(b"plugin_cleanup_string")
        } {
            cleanup_fn(result_ptr);
        }

        Some(InferredProject {
            name: message.name,
            project_dir: PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        })
    }
}

impl MartyPlugin for DylibWorkspaceProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn workspace_provider(&self) -> &dyn WorkspaceProvider {
        self
    }

    fn configuration_options(&self) -> Option<serde_json::Value> {
        self.call_json_function(b"plugin_config_options")
            .unwrap_or_default()
    }
}
