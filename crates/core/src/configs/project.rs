use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::configs::tasks::TaskConfig;
use crate::types::MartyResult;

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub tasks: Option<Vec<TaskConfig>>,
}

pub fn parse_project_config(yaml_str: &str) -> MartyResult<ProjectConfig> {
    let config: ProjectConfig = serde_yaml::from_str(yaml_str)?;
    Ok(config)
}
