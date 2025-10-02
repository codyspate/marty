use thiserror::Error;

/// The main error type for Marty operations
#[derive(Debug, Error)]
pub enum MartyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Task error: {0}")]
    Task(String),

    #[error("Project error: {0}")]
    Project(String),

    #[error("Path error: {0}")]
    Path(String),
}

/// Result type alias for Marty operations
pub type MartyResult<T> = Result<T, MartyError>;
