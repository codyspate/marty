use anyhow::{Context, Result};
use marty_core::{
    platform::PlatformInfo, plugin_cache::PluginCache, workspace_manager::WorkspaceManager,
};
use std::path::Path;

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
            let plugin_configs = manager
                .workspace_config
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
        PluginCommands::Validate { path, name } => {
            validate_plugin(&path, name.as_deref()).await?;
        }
        PluginCommands::CheckRelease {
            github_repo,
            plugin,
            version,
        } => {
            check_release(&github_repo, plugin.as_deref(), &version).await?;
        }
        PluginCommands::ReleaseGuide {
            name,
            version,
            github_repo,
            monorepo,
        } => {
            generate_release_guide(&name, &version, github_repo.as_deref(), monorepo)?;
        }
    }

    Ok(())
}

async fn validate_plugin(path: &Path, expected_name: Option<&str>) -> Result<()> {
    println!("🔍 Validating plugin: {}", path.display());
    println!();

    // Check file exists
    if !path.exists() {
        anyhow::bail!("❌ Plugin file does not exist: {}", path.display());
    }

    // Check file extension matches platform
    let platform = PlatformInfo::current();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext != platform.extension {
        println!(
            "⚠️  Warning: File extension '{}' doesn't match current platform extension '{}'",
            ext, platform.extension
        );
        println!(
            "   Current platform: {} ({})",
            platform.target, platform.extension
        );
    } else {
        println!("✅ File extension matches platform: .{}", ext);
    }

    // Try to load the plugin and get its name
    println!();
    println!("🔌 Loading plugin to check name...");

    // We can't directly load the plugin here without FFI, so provide guidance
    println!("   To fully validate, ensure your plugin's MartyPlugin::name() method returns the expected name.");

    if let Some(expected) = expected_name {
        println!();
        println!("📝 Expected plugin name: {}", expected);
        println!(
            "   ⚠️  Make sure MartyPlugin::name() returns exactly: \"{}\"",
            expected
        );

        // Check if filename follows convention
        let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        if filename.starts_with("marty_plugin_") || filename.starts_with("libmarty_plugin_") {
            let name_from_file = filename
                .trim_start_matches("lib")
                .trim_start_matches("marty_plugin_");

            if name_from_file == expected {
                println!("✅ Filename convention matches expected name");
            } else {
                println!(
                    "⚠️  Filename suggests plugin name: '{}' (expected: '{}')",
                    name_from_file, expected
                );
            }
        }
    }

    println!();
    println!("📋 Binary Naming Convention Checklist:");
    println!("   For release, your binary should be named:");
    if let Some(name) = expected_name {
        println!(
            "   marty-plugin-{}-v{{VERSION}}-{}.{}",
            name, platform.target, platform.extension
        );
    } else {
        println!(
            "   marty-plugin-{{NAME}}-v{{VERSION}}-{}.{}",
            platform.target, platform.extension
        );
    }

    println!();
    println!("✅ Basic validation complete!");
    println!();
    println!("Next steps:");
    println!("1. Test the plugin with: marty plugin list (after configuring it)");
    println!("2. Build for all platforms before release");
    println!("3. Use 'marty plugin release-guide' to see release instructions");

    Ok(())
}

async fn check_release(github_repo: &str, plugin_name: Option<&str>, version: &str) -> Result<()> {
    println!("🔍 Checking GitHub release...");
    println!();

    let current_dir = std::env::current_dir()?;
    let cache = PluginCache::new(&current_dir);

    // Construct URLs for all platforms
    let platforms = [
        ("Linux x86_64", "x86_64-unknown-linux-gnu", "so"),
        ("Linux ARM64", "aarch64-unknown-linux-gnu", "so"),
        ("macOS x86_64", "x86_64-apple-darwin", "dylib"),
        ("macOS ARM64", "aarch64-apple-darwin", "dylib"),
        ("Windows x86_64", "x86_64-pc-windows-msvc", "dll"),
        ("Windows ARM64", "aarch64-pc-windows-msvc", "dll"),
    ];

    let (tag, name) = if let Some(plugin) = plugin_name {
        // Monorepo format
        (
            format!("marty-plugin-{}-v{}", plugin, version),
            plugin.to_string(),
        )
    } else {
        // Separate repo format - extract name from repo
        let name = cache.extract_plugin_name_from_repo(github_repo)
            .context("Failed to extract plugin name from repository. Repository should be in format 'owner/marty-plugin-NAME'")?;
        (format!("v{}", version), name)
    };

    println!("📦 Repository: {}", github_repo);
    println!("🏷️  Release tag: {}", tag);
    println!("📋 Plugin name: {}", name);
    println!();
    println!("Checking for binaries on all platforms:");
    println!();

    let client = reqwest::Client::new();
    let mut found_count = 0;
    let mut missing_platforms = Vec::new();

    for (platform_name, target, ext) in platforms {
        let filename = format!("marty-plugin-{}-v{}-{}.{}", name, version, target, ext);
        let url = format!(
            "https://github.com/{}/releases/download/{}/{}",
            github_repo, tag, filename
        );

        // HEAD request to check if file exists
        match client.head(&url).send().await {
            Ok(response) if response.status().is_success() => {
                println!("  ✅ {} - {}", platform_name, filename);
                found_count += 1;
            }
            _ => {
                println!("  ❌ {} - {} (NOT FOUND)", platform_name, filename);
                missing_platforms.push(platform_name);
            }
        }
    }

    println!();
    println!("📊 Summary: {}/6 platforms available", found_count);

    if found_count == 6 {
        println!("✅ All platforms available! Plugin is ready for use.");
    } else if found_count > 0 {
        println!("⚠️  Some platforms missing:");
        for platform in missing_platforms {
            println!("   - {}", platform);
        }
        println!();
        println!("Users on missing platforms won't be able to use this plugin.");
    } else {
        println!("❌ No binaries found. Please check:");
        println!(
            "   1. Release '{}' exists: https://github.com/{}/releases/tag/{}",
            tag, github_repo, tag
        );
        println!("   2. Binaries are named correctly");
        println!("   3. Repository is public or you have access");
    }

    Ok(())
}

fn generate_release_guide(
    name: &str,
    version: &str,
    github_repo: Option<&str>,
    monorepo: bool,
) -> Result<()> {
    println!("📦 Release Guide for Plugin: {}", name);
    println!("Version: {}", version);
    println!();

    let repo = github_repo.unwrap_or("owner/repo");
    let tag = if monorepo {
        format!("marty-plugin-{}-v{}", name, version)
    } else {
        format!("v{}", version)
    };

    println!("1️⃣  Build for all platforms:");
    println!();
    println!("   # Linux x86_64");
    println!("   cargo build --release --target x86_64-unknown-linux-gnu");
    println!();
    println!("   # Linux ARM64");
    println!("   cargo build --release --target aarch64-unknown-linux-gnu");
    println!();
    println!("   # macOS x86_64");
    println!("   cargo build --release --target x86_64-apple-darwin");
    println!();
    println!("   # macOS ARM64 (Apple Silicon)");
    println!("   cargo build --release --target aarch64-apple-darwin");
    println!();
    println!("   # Windows x86_64");
    println!("   cargo build --release --target x86_64-pc-windows-msvc");
    println!();
    println!("   # Windows ARM64");
    println!("   cargo build --release --target aarch64-pc-windows-msvc");
    println!();

    println!("2️⃣  Rename binaries:");
    println!();
    let binaries = [
        (
            "x86_64-unknown-linux-gnu",
            "so",
            "target/x86_64-unknown-linux-gnu/release/libmarty_plugin_{}.so",
        ),
        (
            "aarch64-unknown-linux-gnu",
            "so",
            "target/aarch64-unknown-linux-gnu/release/libmarty_plugin_{}.so",
        ),
        (
            "x86_64-apple-darwin",
            "dylib",
            "target/x86_64-apple-darwin/release/libmarty_plugin_{}.dylib",
        ),
        (
            "aarch64-apple-darwin",
            "dylib",
            "target/aarch64-apple-darwin/release/libmarty_plugin_{}.dylib",
        ),
        (
            "x86_64-pc-windows-msvc",
            "dll",
            "target/x86_64-pc-windows-msvc/release/marty_plugin_{}.dll",
        ),
        (
            "aarch64-pc-windows-msvc",
            "dll",
            "target/aarch64-pc-windows-msvc/release/marty_plugin_{}.dll",
        ),
    ];

    for (target, ext, source_pattern) in binaries {
        let source = source_pattern.replace("{}", name);
        let dest = format!("marty-plugin-{}-v{}-{}.{}", name, version, target, ext);
        println!("   mv {} {}", source, dest);
    }
    println!();

    println!("3️⃣  Create GitHub Release:");
    println!();
    println!("   Tag: {}", tag);
    println!("   Title: {} v{}", name, version);
    println!();
    println!("   Or using GitHub CLI:");
    println!("   gh release create {} \\", tag);
    for (target, ext, _) in binaries {
        println!(
            "     marty-plugin-{}-v{}-{}.{} \\",
            name, version, target, ext
        );
    }
    println!("     --title \"{} v{}\"", name, version);
    println!();

    println!("4️⃣  Configuration for users:");
    println!();
    if monorepo {
        println!("   plugins:");
        println!("     - githubRepo: {}", repo);
        println!("       plugin: {}", name);
        println!("       version: {}", version);
    } else {
        println!("   plugins:");
        println!("     - githubRepo: {}", repo);
        println!("       version: {}", version);
        println!();
        println!(
            "   Note: Repository should be named 'marty-plugin-{}'",
            name
        );
    }
    println!();

    println!("5️⃣  Validate release:");
    println!();
    println!("   marty plugin check-release \\");
    println!("     --github-repo {} \\", repo);
    if monorepo {
        println!("     --plugin {} \\", name);
    }
    println!("     --version {}", version);
    println!();

    println!("📚 For more details, see: docs/PLUGIN_MONOREPO_APPROACH.md");

    Ok(())
}
