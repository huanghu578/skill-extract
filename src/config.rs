use serde::Deserialize;

/// A parsed skill configuration, ready to be compiled into a [`Skill`](crate::Skill).
///
/// This is the deserialized form of a skill TOML file. Use
/// [`Skill::from_config`](crate::Skill::from_config) to compile it.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillConfig {
    /// Unique skill name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional trigger words for routing (e.g. `["小dropper", "吊弦"]`).
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Extraction rules. Must not be empty.
    pub extractors: Vec<ExtractorConfig>,
    /// Optional output schema for validation.
    #[serde(default)]
    pub output_schema: Option<OutputSchema>,
}

/// A single extraction rule.
///
/// The `pattern` field must contain a `(?P<value>...)` named capture group.
/// The captured text becomes the value associated with `key`.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractorConfig {
    /// Output field name. Supports any Unicode (Chinese, Japanese, etc.).
    pub key: String,
    /// Optional ASCII alias for cross-system integration.
    #[serde(default)]
    pub key_en: Option<String>,
    /// Regex pattern. Must contain a `(?P<value>...)` named capture group.
    pub pattern: String,
    /// If true and no match is found, the key is reported as missing.
    #[serde(default)]
    pub required: bool,
    /// If true, all matches are collected. If false, only the first match.
    #[serde(default = "default_true")]
    pub multi: bool,
    /// Ordered post-processing pipeline.
    #[serde(default)]
    pub postprocess: Vec<PostProcess>,
}

fn default_true() -> bool {
    true
}

/// A built-in value transformation.
///
/// Post-processors run in the order they appear in
/// [`ExtractorConfig::postprocess`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostProcess {
    /// Strip leading and trailing whitespace.
    Trim,
    /// Convert to lowercase.
    Lowercase,
    /// Convert to uppercase.
    Uppercase,
    /// Remove paired `"..."` or `'...'`.
    StripQuotes,
    /// Replace `\` with `/`.
    NormalizePath,
    /// Regex replace.
    Replace {
        /// Regex to search for.
        pattern: String,
        /// Replacement string.
        replacement: String,
    },
    /// Drop values matching the given regex.
    FilterOut {
        /// Regex that, if matched, causes the value to be dropped.
        pattern: String,
    },
}

/// Output schema declaration.
///
/// Declares expected types for extracted keys. After extraction, the
/// framework validates the result and reports mismatches in
/// [`ExtractResult::errors`](crate::ExtractResult::errors).
#[derive(Debug, Clone, Deserialize)]
pub struct OutputSchema {
    /// Field name to type declaration.
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, FieldSpec>,
}

/// Type declaration for a single output field.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldSpec {
    /// Expected type: `"string"`, `"path"`, `"int"`, `"float"`, `"bool"`.
    #[serde(rename = "type", default = "default_type")]
    pub kind: String,
    /// If true, the field must be present after extraction.
    #[serde(default)]
    pub required: bool,
}

fn default_type() -> String {
    "string".to_string()
}
