use anyhow::Result;
use marty_core::{plugin_cache::PluginCache, workspace_manager::WorkspaceManager};

use crate::PluginCommands;

pub async fn execute(manager: &WorkspaceManager, command: PluginCommands) -> Result<()> {
    let cache = PluginCache::new(&manager.workspace.root);

    match command {
        PluginCommands::List => {
            let cached_plugins = cache.list_cached_plugins()?;
            if cached_plugins.is_empty() {
                println!("No cached plugins found.");
            } else {
                println!("Cached plugins:");
                for (name, path) in cached_plugins {
                    println!("  {} -> {}", name, path.display());
                }
            }
        }
        PluginCommands::Clear => {
            cache.clear_cache().await?;
            println!("Plugin cache cleared successfully.");
        }
        PluginCommands::Update => {
            // Re-initialize the workspace manager to force plugin re-download
            println!("Clearing plugin cache...");
            cache.clear_cache().await?;
            
            println!("Re-downloading plugins...");
            // The plugins will be re-downloaded when the workspace manager is re-initialized
            let plugin_configs = manager.workspace_config
                .plugins
                .as_ref()
                .cloned()
                .unwrap_or_default();
            
            let cached_plugins = cache.resolve_plugins(&plugin_configs).await?;
            
            println!("Updated {} plugin(s):", cached_plugins.len());
            for plugin in cached_plugins {
                if let Some(url) = &plugin.url {
                    println!("  {} <- {}", plugin.name, url);
                } else {
                    println!("  {} (local)", plugin.name);
                }
            }
        }
    }

    Ok(())
}