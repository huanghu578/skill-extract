use serde::Deserialize;

/// A single skill definition loaded from a TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillConfig {
    /// Unique skill name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional trigger words for routing (e.g. ["小dropper", "吊弦"]).
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Extraction rules.
    pub extractors: Vec<ExtractorConfig>,
    /// Optional output schema for validation.
    #[serde(default)]
    pub output_schema: Option<OutputSchema>,
}

/// A single extraction rule.
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

/// Built-in post-processing operations.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostProcess {
    Trim,
    Lowercase,
    Uppercase,
    StripQuotes,
    NormalizePath,
    Replace { pattern: String, replacement: String },
    FilterOut { pattern: String },
}

/// Output schema declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputSchema {
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, FieldSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldSpec {
    /// Expected type: "string", "path", "int", "float", "bool".
    #[serde(rename = "type", default = "default_type")]
    pub kind: String,
    #[serde(default)]
    pub required: bool,
}

fn default_type() -> String {
    "string".to_string()
}