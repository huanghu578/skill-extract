use thiserror::Error;

/// Errors produced while loading or compiling a skill.
///
/// Every variant carries context — file paths, key names, expected vs.
/// actual types — so callers can handle failures programmatically rather
/// than parsing strings.
///
/// Note: [`Skill::extract`](crate::Skill::extract) does **not** return
/// `SkillError`. Extraction-time issues (missing keys, schema violations)
/// are reported inside [`ExtractResult`](crate::ExtractResult) instead.
#[derive(Debug, Error)]
pub enum SkillError {
    /// The TOML file could not be found.
    #[error("config file not found: {path} (cwd: {cwd:?})")]
    ConfigNotFound { path: String, cwd: Option<String> },

    /// Reading the TOML file failed for a reason other than "not found".
    #[error("failed to read config [{path}]: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// The TOML content could not be parsed.
    #[error("invalid TOML in [{path}]: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    /// A regex pattern failed to compile.
    #[error("invalid pattern for key [{key}]: {reason}")]
    InvalidPattern { key: String, reason: String },

    /// A regex pattern lacks the required `(?P<value>...)` capture group.
    #[error("pattern for key [{key}] must contain a (?P<value>...) named capture group")]
    MissingValueGroup { key: String },

    /// Schema validation: a required key was not found.
    #[error("missing required keys: {keys:?}")]
    MissingRequired { keys: Vec<String> },

    /// Schema validation: a value does not match its declared type.
    #[error("type mismatch for key [{key}]: expected {expected}, got {got}")]
    TypeMismatch {
        key: String,
        expected: String,
        got: String,
    },

    /// Structural problem with the configuration itself.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

/// Convenience alias for `Result<T, SkillError>`.
pub type Result<T> = std::result::Result<T, SkillError>;
