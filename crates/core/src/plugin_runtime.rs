use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::workspace::{InferredProject, Workspace, WorkspaceProvider};
use anyhow::{Context, Result};
use marty_plugin_protocol::InferredProjectMessage;
use serde_json::Value;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::WasiCtx;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::sync::add_to_linker;

struct WasiState {
    wasi: WasiCtx,
}

pub struct WasmWorkspaceProvider {
    name: String,
    engine: Engine,
    module: Module,
    stdout_lock: Mutex<()>,
}

impl WasmWorkspaceProvider {
    /// Scan the .marty/plugins directory for .wasm files and return providers for each
    pub fn load_all_from_plugins_dir() -> Result<Vec<Self>> {
        let plugins_dir = PathBuf::from(".marty/plugins");
        if !plugins_dir.exists() {
            return Ok(Vec::new());
        }
        let mut providers = Vec::new();
        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "wasm").unwrap_or(false) {
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                match WasmWorkspaceProvider::from_wasm(name, path.clone()) {
                    Ok(provider) => providers.push(provider),
                    Err(e) => eprintln!("Failed to load plugin '{}': {e}", path.display()),
                }
            }
        }
        Ok(providers)
    }

    pub fn from_wasm(name: &str, wasm_path: PathBuf) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, &wasm_path)
            .with_context(|| format!("Failed to load wasm module at {}", wasm_path.display()))?;

        Ok(Self {
            name: name.to_string(),
            engine,
            module,
            stdout_lock: Mutex::new(()),
        })
    }

    fn run_command(&self, args: &[&str], stdin_data: Option<&[u8]>) -> Result<String> {
        let _guard = self.stdout_lock.lock().expect("stdout mutex poisoned");

        let stdout = WritePipe::new_in_memory();
        let stderr = WritePipe::new_in_memory();
        let stdin = stdin_data
            .map(|data| ReadPipe::from(data.to_vec()))
            .unwrap_or_else(|| ReadPipe::from(Vec::new()));

        let mut builder = WasiCtxBuilder::new();
        builder.inherit_env()?;

        let mut argv = Vec::with_capacity(args.len() + 1);
        argv.push(self.name.clone());
        argv.extend(args.iter().map(|s| s.to_string()));
        builder.args(&argv)?;
        builder.stdin(Box::new(stdin));
        builder.stdout(Box::new(stdout.clone()));
        builder.stderr(Box::new(stderr.clone()));
        let wasi = builder.build();

        let mut linker = Linker::new(&self.engine);
        add_to_linker(&mut linker, |state: &mut WasiState| &mut state.wasi)?;

        {
            let mut store = Store::new(&self.engine, WasiState { wasi });

            let instance = linker
                .instantiate(&mut store, &self.module)
                .context("Failed to instantiate plugin module")?;

            let start = instance
                .get_typed_func::<(), ()>(&mut store, "_start")
                .context("Plugin module missing _start entry point")?;
            start
                .call(&mut store, ())
                .context("Plugin execution failed")?;
            // store and wasi dropped here
        }

        let stdout_buf = stdout
            .try_into_inner()
            .map_err(|_| anyhow::anyhow!("Failed to retrieve plugin stdout"))?
            .into_inner();

        let output = String::from_utf8(stdout_buf).context("Plugin stdout was not valid UTF-8")?;

        Ok(output)
    }
}

impl WorkspaceProvider for WasmWorkspaceProvider {
    fn include_path_globs(&self) -> Vec<String> {
        // Try new include-globs command first
        match self.run_command(&["include-globs"], None) {
            Ok(output) => {
                if let Ok(includes) = serde_json::from_str::<Vec<String>>(&output) {
                    return includes;
                }
            }
            Err(_) => {
                // Fall back to trying ignore-globs for backward compatibility
                // If a plugin doesn't support include-globs, return empty to use defaults
            }
        }
        
        // Return empty vector to use default includes
        Vec::new()
    }

    fn on_file_found(&self, _workspace: &Workspace, path: &Path) -> Option<InferredProject> {
        let contents = fs::read_to_string(path).ok()?;
        let path_arg = path.as_os_str().to_string_lossy().to_string();
        let args = ["on-file-found", path_arg.as_str()];

        let output = self.run_command(&args, Some(contents.as_bytes())).ok()?;
        if output.trim().is_empty() {
            return None;
        }

        let value: Value = serde_json::from_str(&output).ok()?;
        if value.is_null() {
            return None;
        }

        let message: InferredProjectMessage = serde_json::from_value(value).ok()?;
        Some(InferredProject {
            name: message.name,
            project_dir: PathBuf::from(message.project_dir),
            discovered_by: message.discovered_by,
            workspace_dependencies: message.workspace_dependencies,
        })
    }
}

// Plugin WASM artifact discovery and build logic removed. Plugins are now loaded from .marty/plugins/*.wasm only.
