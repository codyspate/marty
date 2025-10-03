use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use reqwest;
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::configs::workspace::PluginConfig;
use crate::platform::PlatformInfo;
use crate::plugin_runtime_dylib::DylibWorkspaceProvider;
use crate::types::MartyResult;
use marty_plugin_protocol::MartyPlugin;
use serde_json::Value;

/// Manages downloading, caching, and loading of dynamic library plugins
pub struct PluginCache {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

/// Information about a cached plugin
#[derive(Debug, Clone)]
pub struct CachedPlugin {
    pub name: String,
    pub path: PathBuf,
    pub url: Option<String>,
    pub enabled: bool,
    pub options: Option<serde_json::Value>,
}

impl PluginCache {
    /// Create a new plugin cache instance
    pub fn new(workspace_root: &Path) -> Self {
        let cache_dir = workspace_root.join(".marty").join("cache").join("plugins");
        let client = reqwest::Client::new();

        Self { cache_dir, client }
    }

    /// Ensure the cache directory exists
    pub async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .with_context(|| {
                format!(
                    "Failed to create plugin cache directory: {}",
                    self.cache_dir.display()
                )
            })?;
        Ok(())
    }

    /// Process plugin configurations and return cached plugin information
    pub async fn resolve_plugins(
        &self,
        plugin_configs: &[PluginConfig],
    ) -> MartyResult<Vec<CachedPlugin>> {
        self.initialize().await.map_err(|e| {
            crate::types::MartyError::Config(format!("Failed to initialize plugin cache: {}", e))
        })?;

        let mut cached_plugins = Vec::new();

        for config in plugin_configs {
            let enabled = config.enabled.unwrap_or(true);

            if !enabled {
                continue;
            }

            let cached_plugin = self.resolve_plugin(config).await.map_err(|e| {
                crate::types::MartyError::Config(format!("Failed to resolve plugin: {}", e))
            })?;

            cached_plugins.push(cached_plugin);
        }

        Ok(cached_plugins)
    }

    /// Load a dynamic library plugin and extract its name from the MartyPlugin implementation
    fn load_plugin_and_get_name(&self, dylib_path: &Path) -> Result<String> {
        let plugin = DylibWorkspaceProvider::from_dylib_with_temp_copy(dylib_path.to_path_buf())?;
        Ok(plugin.name().to_string())
    }

    /// Load a dynamic library plugin and validate options against its configuration schema
    fn load_plugin_and_validate_options(
        &self,
        dylib_path: &Path,
        options: &Option<Value>,
    ) -> Result<String> {
        let plugin = DylibWorkspaceProvider::from_dylib_with_temp_copy(dylib_path.to_path_buf())?;

        // Validate options if provided
        if let Some(options_value) = options {
            self.validate_plugin_options(&plugin, options_value)?;
        }

        Ok(plugin.name().to_string())
    }

    /// Validate plugin options against the plugin's configuration schema
    fn validate_plugin_options(
        &self,
        plugin: &DylibWorkspaceProvider,
        options: &Value,
    ) -> Result<()> {
        if let Some(schema_value) = plugin.configuration_options() {
            println!(
                "Validating options for plugin '{}' against schema",
                plugin.name()
            );
            // Convert the JSON schema value to a validation function
            // For now, we'll do basic validation - this could be enhanced with a proper JSON Schema validator
            self.validate_options_against_schema(options, &schema_value, plugin.name())?;
        } else {
            println!(
                "Plugin '{}' has no configuration schema - skipping validation",
                plugin.name()
            );
        }
        Ok(())
    }

    /// Basic validation of options against a JSON schema
    fn validate_options_against_schema(
        &self,
        options: &Value,
        schema: &Value,
        plugin_name: &str,
    ) -> Result<()> {
        // Extract properties from schema if it exists
        if let Some(schema_obj) = schema.as_object() {
            if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
                if let Some(options_obj) = options.as_object() {
                    // Check for unknown properties if additionalProperties is false
                    if let Some(additional_props) = schema_obj.get("additionalProperties") {
                        if additional_props == &Value::Bool(false) {
                            for key in options_obj.keys() {
                                if !properties.contains_key(key) {
                                    return Err(anyhow::anyhow!(
                                        "Plugin '{}' does not support option '{}'. Valid options are: {}",
                                        plugin_name,
                                        key,
                                        properties.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
                                    ));
                                }
                            }
                        }
                    }

                    // Basic type validation for each property
                    for (key, value) in options_obj {
                        if let Some(prop_schema) = properties.get(key).and_then(|p| p.as_object()) {
                            if let Some(expected_type) =
                                prop_schema.get("type").and_then(|t| t.as_str())
                            {
                                let actual_type = match value {
                                    Value::String(_) => "string",
                                    Value::Number(_) => "number",
                                    Value::Bool(_) => "boolean",
                                    Value::Array(_) => "array",
                                    Value::Object(_) => "object",
                                    Value::Null => "null",
                                };

                                if expected_type != actual_type {
                                    return Err(anyhow::anyhow!(
                                        "Plugin '{}' option '{}' expects type '{}', got '{}'",
                                        plugin_name,
                                        key,
                                        expected_type,
                                        actual_type
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "Plugin '{}' options must be an object, got {}",
                        plugin_name,
                        options
                    ));
                }
            }
        }
        Ok(())
    }

    /// Resolve a single plugin configuration to a cached plugin
    async fn resolve_plugin(&self, config: &PluginConfig) -> Result<CachedPlugin> {
        let enabled = config.enabled.unwrap_or(true);
        let options = config.options.clone();

        // Priority 1: GitHub repository + version (new convention)
        if let (Some(github_repo), Some(version)) = (&config.github_repo, &config.version) {
            let (url, temp_name) = if let Some(plugin_name) = &config.plugin {
                // Monorepo mode: plugin specified separately
                let url =
                    self.resolve_github_plugin_url_monorepo(github_repo, plugin_name, version)?;
                (url, plugin_name.clone())
            } else {
                // Separate repo mode: extract plugin name from repository name
                let url = self.resolve_github_plugin_url(github_repo, version)?;
                let temp_name = self.extract_plugin_name_from_repo(github_repo)?;
                (url, temp_name)
            };

            let cached_path = self.download_and_cache_plugin(&temp_name, &url).await?;

            let plugin_name = self
                .load_plugin_and_validate_options(&cached_path, &options)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: Failed to validate plugin options for '{}': {}",
                        temp_name, e
                    );
                    self.load_plugin_and_get_name(&cached_path)
                        .unwrap_or_else(|_| temp_name.clone())
                });

            return Ok(CachedPlugin {
                name: plugin_name,
                path: cached_path,
                url: Some(url),
                enabled,
                options,
            });
        }

        // Priority 2: Direct URL (fallback for custom hosting)
        if let Some(url) = &config.url {
            // Use temporary name for downloading (extract from URL)
            let temp_name = url
                .rsplit('/')
                .next()
                .unwrap_or("unnamed")
                // Remove various possible dynamic library extensions
                .trim_end_matches(".so")
                .trim_end_matches(".dylib")
                .trim_end_matches(".dll")
                .to_string();

            let cached_path = self.download_and_cache_plugin(&temp_name, url).await?;

            // Load the plugin, get its name, and validate options
            let plugin_name = self
                .load_plugin_and_validate_options(&cached_path, &options)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: Failed to validate plugin options for '{}': {}",
                        temp_name, e
                    );
                    self.load_plugin_and_get_name(&cached_path)
                        .unwrap_or_else(|_| temp_name.clone())
                });

            return Ok(CachedPlugin {
                name: plugin_name,
                path: cached_path,
                url: Some(url.clone()),
                enabled,
                options,
            });
        }

        // Priority 3: Local path
        if let Some(path_str) = &config.path {
            let temp_name = Path::new(path_str)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string();

            let path = if path_str == "builtin" {
                // Handle builtin plugins - look for them in the .marty/cache/plugins directory first
                let extension = PlatformInfo::current_extension();

                let cache_path = self
                    .cache_dir
                    .join(format!("marty-plugin-{}.{}", temp_name, extension));
                if cache_path.exists() {
                    cache_path
                } else {
                    // Fallback to .marty/plugins directory
                    self.cache_dir
                        .parent()
                        .unwrap()
                        .join("plugins")
                        .join(format!("{}.{}", temp_name, extension))
                }
            } else {
                PathBuf::from(path_str)
            };

            // Load the plugin, get its name, and validate options
            let plugin_name = self
                .load_plugin_and_validate_options(&path, &options)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: Failed to validate plugin options for '{}': {}",
                        temp_name, e
                    );
                    self.load_plugin_and_get_name(&path)
                        .unwrap_or_else(|_| temp_name.clone())
                });

            return Ok(CachedPlugin {
                name: plugin_name,
                path,
                url: None,
                enabled,
                options,
            });
        }

        // If neither repository+version, URL, nor path is specified, this is an error
        Err(anyhow::anyhow!(
            "Plugin configuration must specify either 'repository' + 'version', 'url', or 'path'"
        ))
    }

    /// Resolve a GitHub plugin repository and version to a download URL
    /// For separate plugin repositories (e.g., "owner/marty-plugin-cargo")
    fn resolve_github_plugin_url(&self, repository: &str, version: &str) -> Result<String> {
        let plugin_name = self.extract_plugin_name_from_repo(repository)?;
        let platform = PlatformInfo::current();

        // Standard naming convention: marty-plugin-{name}-v{version}-{target}.{ext}
        let filename = format!(
            "marty-plugin-{}-v{}-{}.{}",
            plugin_name, version, platform.target, platform.extension
        );

        // GitHub releases URL format
        let url = format!(
            "https://github.com/{}/releases/download/v{}/{}",
            repository, version, filename
        );

        Ok(url)
    }

    /// Resolve a GitHub plugin URL for monorepo containing multiple plugins
    /// For repositories that contain multiple plugins (e.g., "owner/marty")
    fn resolve_github_plugin_url_monorepo(
        &self,
        repository: &str,
        plugin_name: &str,
        version: &str,
    ) -> Result<String> {
        let platform = PlatformInfo::current();

        // Monorepo naming convention: marty-plugin-{name}-v{version}-{target}.{ext}
        let filename = format!(
            "marty-plugin-{}-v{}-{}.{}",
            plugin_name, version, platform.target, platform.extension
        );

        // GitHub releases URL format with plugin name in tag
        let url = format!(
            "https://github.com/{}/releases/download/marty-plugin-{}-v{}/{}",
            repository, plugin_name, version, filename
        );

        Ok(url)
    }
    /// Extract plugin name from GitHub repository
    /// Expected format: "owner/marty-plugin-name" -> "name"
    pub fn extract_plugin_name_from_repo(&self, repository: &str) -> Result<String> {
        let repo_name = repository.rsplit('/').next().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid repository format: '{}'. Expected 'owner/repo'",
                repository
            )
        })?;

        let plugin_name = repo_name
            .strip_prefix("marty-plugin-")
            .ok_or_else(|| anyhow::anyhow!(
                "Repository name must start with 'marty-plugin-': '{}'. Example: 'owner/marty-plugin-cargo'", 
                repo_name
            ))?;

        if plugin_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Plugin name cannot be empty. Repository format should be 'owner/marty-plugin-{{name}}'"
            ));
        }

        Ok(plugin_name.to_string())
    }

    /// Download a plugin from URL and cache it locally
    async fn download_and_cache_plugin(&self, name: &str, url: &str) -> Result<PathBuf> {
        let url_hash = format!("{:x}", Sha256::digest(url.as_bytes()));

        // Determine file extension based on platform
        let extension = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        let cache_filename = format!("{}_{}.{}", name, &url_hash[..8], extension);
        let cache_path = self.cache_dir.join(&cache_filename);

        // Check if already cached
        if cache_path.exists() {
            println!("Using cached plugin: {} ({})", name, cache_filename);
            return Ok(cache_path);
        }

        println!("Downloading plugin '{}' from {}", name, url);

        // Download the plugin
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to download plugin from {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = if status == reqwest::StatusCode::NOT_FOUND {
                format!(
                    "Failed to download plugin from {}: HTTP 404 Not Found\n\n\
                    Possible causes:\n\
                    • Plugin version '{}' may not exist or hasn't been published\n\
                    • The binary for your platform ({}) may not be available\n\
                    • The repository may not have separate plugin releases yet\n\n\
                    Suggestions:\n\
                    • Check available releases: https://github.com/{}/releases\n\
                    • Verify the version number is correct\n\
                    • Try using a direct URL to the main repository if plugins aren't published separately\n\
                    • For development, use a local path: path: \"/path/to/plugin.so\"",
                    url,
                    name,
                    PlatformInfo::current().target,
                    // Extract owner/repo from URL if possible
                    url.split("/releases/").next().unwrap_or("").trim_start_matches("https://github.com/")
                )
            } else {
                format!("Failed to download plugin from {}: HTTP {}", url, status)
            };
            return Err(anyhow::anyhow!(error_msg));
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read plugin data from {}", url))?;

        // Basic validation - check if it's a binary file (not empty)
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Downloaded file from {} is empty", url));
        }

        // Write to cache
        let mut file = File::create(&cache_path)
            .await
            .with_context(|| format!("Failed to create cache file: {}", cache_path.display()))?;

        file.write_all(&bytes).await.with_context(|| {
            format!("Failed to write plugin to cache: {}", cache_path.display())
        })?;

        file.flush().await.with_context(|| {
            format!(
                "Failed to flush plugin cache file: {}",
                cache_path.display()
            )
        })?;

        println!("Cached plugin: {} -> {}", name, cache_filename);
        Ok(cache_path)
    }

    /// Get a list of all cached plugins on disk (for cleanup purposes)
    pub fn list_cached_plugins(&self) -> Result<HashMap<String, PathBuf>> {
        let mut plugins = HashMap::new();

        if !self.cache_dir.exists() {
            return Ok(plugins);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check for dynamic library extensions
            let is_dylib = path
                .extension()
                .map(|e| e == "so" || e == "dylib" || e == "dll")
                .unwrap_or(false);

            if is_dylib {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    plugins.insert(filename.to_string(), path);
                }
            }
        }

        Ok(plugins)
    }

    /// Clear the plugin cache
    pub async fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            tokio::fs::remove_dir_all(&self.cache_dir)
                .await
                .with_context(|| {
                    format!("Failed to clear plugin cache: {}", self.cache_dir.display())
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plugin_name_from_repo() {
        let cache = PluginCache {
            cache_dir: PathBuf::from("/tmp"),
            client: reqwest::Client::new(),
        };

        // Valid repository names
        assert_eq!(
            cache
                .extract_plugin_name_from_repo("codyspate/marty-plugin-cargo")
                .unwrap(),
            "cargo"
        );
        assert_eq!(
            cache
                .extract_plugin_name_from_repo("user/marty-plugin-typescript")
                .unwrap(),
            "typescript"
        );
        assert_eq!(
            cache
                .extract_plugin_name_from_repo("org/marty-plugin-my-custom-plugin")
                .unwrap(),
            "my-custom-plugin"
        );

        // Invalid formats
        assert!(cache.extract_plugin_name_from_repo("no-slash").is_err());
        assert!(cache
            .extract_plugin_name_from_repo("user/cargo-plugin")
            .is_err());
        assert!(cache
            .extract_plugin_name_from_repo("user/marty-plugin-")
            .is_err());
    }

    #[test]
    fn test_resolve_github_plugin_url_linux() {
        let cache = PluginCache {
            cache_dir: PathBuf::from("/tmp"),
            client: reqwest::Client::new(),
        };

        let url = cache
            .resolve_github_plugin_url("codyspate/marty-plugin-cargo", "0.2.0")
            .unwrap();

        // URL should contain the repository, version, and platform info
        assert!(url.contains("github.com/codyspate/marty-plugin-cargo"));
        assert!(url.contains("v0.2.0"));
        assert!(url.contains("marty-plugin-cargo-v0.2.0"));

        // Should have platform-specific target and extension
        #[cfg(target_os = "linux")]
        {
            assert!(
                url.contains("x86_64-unknown-linux-gnu")
                    || url.contains("aarch64-unknown-linux-gnu")
            );
            assert!(url.ends_with(".so"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(url.contains("apple-darwin"));
            assert!(url.ends_with(".dylib"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(url.contains("pc-windows-msvc"));
            assert!(url.ends_with(".dll"));
        }
    }

    #[test]
    fn test_resolve_github_plugin_url_format() {
        let cache = PluginCache {
            cache_dir: PathBuf::from("/tmp"),
            client: reqwest::Client::new(),
        };

        let url = cache
            .resolve_github_plugin_url("user/marty-plugin-test", "1.0.0")
            .unwrap();

        // Verify URL structure
        assert!(
            url.starts_with("https://github.com/user/marty-plugin-test/releases/download/v1.0.0/")
        );
        assert!(url.contains("marty-plugin-test-v1.0.0-"));
    }

    #[test]
    fn test_resolve_github_plugin_url_monorepo() {
        let cache = PluginCache {
            cache_dir: PathBuf::from("/tmp"),
            client: reqwest::Client::new(),
        };

        let url = cache
            .resolve_github_plugin_url_monorepo("codyspate/marty", "typescript", "0.2.2")
            .unwrap();

        // URL should contain the repository, plugin name, and version
        assert!(url.contains("github.com/codyspate/marty"));
        assert!(url.contains("marty-plugin-typescript-v0.2.2"));

        // Tag should be marty-plugin-{name}-v{version}
        assert!(url.contains("/releases/download/marty-plugin-typescript-v0.2.2/"));

        // Should have platform-specific target and extension
        #[cfg(target_os = "linux")]
        {
            assert!(
                url.contains("x86_64-unknown-linux-gnu")
                    || url.contains("aarch64-unknown-linux-gnu")
            );
            assert!(url.ends_with(".so"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(url.contains("apple-darwin"));
            assert!(url.ends_with(".dylib"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(url.contains("pc-windows-msvc"));
            assert!(url.ends_with(".dll"));
        }
    }
}
