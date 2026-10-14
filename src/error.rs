use thiserror::Error;

/// Structured error model for the extraction framework.
#[derive(Debug, Error)]
pub enum SkillError {
    #[error("config file not found: {path} (cwd: {cwd:?})")]
    ConfigNotFound { path: String, cwd: Option<String> },

    #[error("failed to read config [{path}]: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid TOML in [{path}]: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("invalid pattern for key [{key}]: {reason}")]
    InvalidPattern { key: String, reason: String },

    #[error("pattern for key [{key}] must contain a (?P<value>...) named capture group")]
    MissingValueGroup { key: String },

    #[error("missing required keys: {keys:?}")]
    MissingRequired { keys: Vec<String> },

    #[error("type mismatch for key [{key}]: expected {expected}, got {got}")]
    TypeMismatch {
        key: String,
        expected: String,
        got: String,
    },

    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, SkillError>;