use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InferredProjectMessage {
    pub name: String,
    pub project_dir: String,
    pub discovered_by: String,
    pub workspace_dependencies: Vec<String>,
}

impl InferredProjectMessage {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        project_dir: impl Into<String>,
        discovered_by: impl Into<String>,
        workspace_dependencies: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            project_dir: project_dir.into(),
            discovered_by: discovered_by.into(),
            workspace_dependencies,
        }
    }
}
