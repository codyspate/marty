use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::MartyResult;

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub plugins: Option<Vec<PluginConfig>>,
    /// Glob patterns for paths to include in workspace traversal. If empty or not specified, all paths are included.
    pub includes: Option<Vec<String>>,
    /// Glob patterns for paths to exclude from workspace traversal.
    pub excludes: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginConfig {
    pub url: Option<String>,
    pub path: Option<String>,
    pub enabled: Option<bool>,
    pub options: Option<serde_json::Value>,
}

pub fn parse_workspace_config(yaml_str: &str) -> MartyResult<WorkspaceConfig> {
    let config: WorkspaceConfig = serde_yaml::from_str(yaml_str)?;
    Ok(config)
}
