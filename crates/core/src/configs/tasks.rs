use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::MartyResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Command {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskConfig {
    pub name: String,
    pub description: Option<String>,
    pub script: Option<String>,
    pub command: Option<Command>,
    pub dependencies: Option<Vec<String>>,
    pub override_targets: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TasksFileConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tasks: Vec<TaskConfig>,
    pub tags: Option<Vec<String>>,
    pub targets: Option<Vec<String>>,
}

pub fn parse_tasks_config(yaml_str: &str) -> MartyResult<TasksFileConfig> {
    let config: TasksFileConfig = serde_yaml::from_str(yaml_str)?;
    Ok(config)
}
