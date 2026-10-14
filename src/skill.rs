use crate::config::{ExtractorConfig, PostProcess, SkillConfig};
use crate::error::{Result, SkillError};
use crate::schema::validate_schema;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A compiled extractor with its regex and pipeline.
#[derive(Debug)]
pub(crate) struct CompiledExtractor {
    pub key: String,
    pub key_en: Option<String>,
    pub regex: Regex,
    pub required: bool,
    pub multi: bool,
    pub postprocess: Vec<CompiledPostProcess>,
}

/// Compiled post-processing steps.
#[derive(Debug)]
pub(crate) enum CompiledPostProcess {
    Trim,
    Lowercase,
    Uppercase,
    StripQuotes,
    NormalizePath,
    Replace { regex: Regex, replacement: String },
    FilterOut { regex: Regex },
}

/// The result of an extraction.
#[derive(Debug, Clone, Serialize)]
pub struct ExtractResult {
    /// Successfully extracted key-value pairs.
    pub values: HashMap<String, Vec<String>>,
    /// Required keys that were not found.
    pub missing: Vec<String>,
    /// Errors encountered during extraction or validation.
    pub errors: Vec<String>,
}

impl ExtractResult {
    /// Returns true if no missing keys and no errors.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.errors.is_empty()
    }

    /// Pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
    }

    /// Compact JSON.
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
    }

    /// Get the first value for a key.
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.first()).map(|s| s.as_str())
    }
}

/// A compiled skill ready to extract from text.
#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub triggers: Vec<String>,
    pub(crate) extractors: Vec<CompiledExtractor>,
    pub(crate) schema: Option<crate::config::OutputSchema>,
}

impl Skill {
    /// Load from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SkillError::ConfigNotFound {
                    path: path.display().to_string(),
                    cwd: std::env::current_dir()
                        .ok()
                        .map(|p| p.display().to_string()),
                }
            } else {
                SkillError::Io {
                    path: path.display().to_string(),
                    source: e,
                }
            }
        })?;
        Self::from_toml(&content)
    }

    /// Load from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let config: SkillConfig = toml::from_str(toml_str).map_err(|e| {
            SkillError::Toml {
                path: "<inline>".to_string(),
                source: e,
            }
        })?;
        Self::from_config(config)
    }

    /// Build from a parsed config.
    pub fn from_config(config: SkillConfig) -> Result<Self> {
        if config.extractors.is_empty() {
            return Err(SkillError::InvalidConfig(
                "extractors must not be empty".to_string(),
            ));
        }

        let mut extractors = Vec::with_capacity(config.extractors.len());
        for ec in config.extractors {
            extractors.push(compile_extractor(ec)?);
        }

        Ok(Skill {
            name: config.name,
            description: config.description,
            triggers: config.triggers,
            extractors,
            schema: config.output_schema,
        })
    }

    /// Check whether this skill's triggers match the given text.
    pub fn matches(&self, text: &str) -> bool {
        self.triggers.iter().any(|t| text.contains(t))
    }

    /// Extract all configured keys from `text`.
    pub fn extract(&self, text: &str) -> ExtractResult {
        let mut values: HashMap<String, Vec<String>> = HashMap::new();
        let mut missing: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for ext in &self.extractors {
            let mut found: Vec<String> = Vec::new();

            let push_value = |raw: &str, found: &mut Vec<String>| {
                let mut val = raw.to_string();
                for pp in &ext.postprocess {
                    match pp {
                        CompiledPostProcess::Trim => val = val.trim().to_string(),
                        CompiledPostProcess::Lowercase => val = val.to_lowercase(),
                        CompiledPostProcess::Uppercase => val = val.to_uppercase(),
                        CompiledPostProcess::StripQuotes => val = strip_quotes(&val),
                        CompiledPostProcess::NormalizePath => val = val.replace('\\', "/"),
                        CompiledPostProcess::Replace { regex, replacement } => {
                            val = regex.replace_all(&val, replacement.as_str()).to_string();
                        }
                        CompiledPostProcess::FilterOut { regex } => {
                            if regex.is_match(&val) {
                                return;
                            }
                        }
                    }
                }
                if !val.is_empty() && !found.contains(&val) {
                    found.push(val);
                }
            };

            if ext.multi {
                for caps in ext.regex.captures_iter(text) {
                    if let Some(m) = caps.name("value") {
                        push_value(m.as_str(), &mut found);
                    }
                }
            } else if let Some(caps) = ext.regex.captures(text) {
                if let Some(m) = caps.name("value") {
                    push_value(m.as_str(), &mut found);
                }
            }

            if found.is_empty() {
                if ext.required {
                    missing.push(ext.key.clone());
                }
            } else {
                values.insert(ext.key.clone(), found.clone());
                if let Some(alias) = &ext.key_en {
                    values.insert(alias.clone(), found);
                }
            }
        }

        if let Some(schema) = &self.schema {
            if let Err(e) = validate_schema(&values, schema) {
                errors.push(e.to_string());
            }
        }

        ExtractResult { values, missing, errors }
    }

    /// Extract and return JSON directly.
    pub fn extract_to_json(&self, text: &str) -> String {
        self.extract(text).to_json()
    }
}

fn compile_extractor(ec: ExtractorConfig) -> Result<CompiledExtractor> {
    let regex = Regex::new(&ec.pattern).map_err(|e| SkillError::InvalidPattern {
        key: ec.key.clone(),
        reason: e.to_string(),
    })?;

    if !regex.capture_names().flatten().any(|n| n == "value") {
        return Err(SkillError::MissingValueGroup { key: ec.key.clone() });
    }

    let mut postprocess = Vec::with_capacity(ec.postprocess.len());
    for pp in ec.postprocess {
        postprocess.push(match pp {
            PostProcess::Trim => CompiledPostProcess::Trim,
            PostProcess::Lowercase => CompiledPostProcess::Lowercase,
            PostProcess::Uppercase => CompiledPostProcess::Uppercase,
            PostProcess::StripQuotes => CompiledPostProcess::StripQuotes,
            PostProcess::NormalizePath => CompiledPostProcess::NormalizePath,
            PostProcess::Replace { pattern, replacement } => {
                let regex = Regex::new(&pattern).map_err(|e| SkillError::InvalidPattern {
                    key: ec.key.clone(),
                    reason: format!("replace pattern: {}", e),
                })?;
                CompiledPostProcess::Replace { regex, replacement }
            }
            PostProcess::FilterOut { pattern } => {
                let regex = Regex::new(&pattern).map_err(|e| SkillError::InvalidPattern {
                    key: ec.key.clone(),
                    reason: format!("filter_out pattern: {}", e),
                })?;
                CompiledPostProcess::FilterOut { regex }
            }
        });
    }

    Ok(CompiledExtractor {
        key: ec.key,
        key_en: ec.key_en,
        regex,
        required: ec.required,
        multi: ec.multi,
        postprocess,
    })
}

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}