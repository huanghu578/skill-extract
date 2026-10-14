use crate::error::{Result, SkillError};
use crate::postprocess::PostProcessorRegistry;
use crate::skill::{ExtractResult, Skill};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manages a collection of loaded skills.
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    pub postprocessors: PostProcessorRegistry,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            postprocessors: PostProcessorRegistry::new(),
        }
    }

    /// Load all `.toml` skill files from a directory.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let mut registry = Self::new();
        let dir = dir.as_ref();

        let entries = fs::read_dir(dir).map_err(|e| SkillError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| SkillError::Io {
                path: dir.display().to_string(),
                source: e,
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let skill = Skill::from_file(&path)?;
                registry.register(skill);
            }
        }

        Ok(registry)
    }

    /// Register a single skill.
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all skill names.
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.skills.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Route text to a skill by trigger word. Returns the first match.
    pub fn route(&self, text: &str) -> Option<&Skill> {
        self.skills.values().find(|s| s.matches(text))
    }

    /// Extract from text using the routed skill.
    pub fn extract_routed(&self, text: &str) -> Option<ExtractResult> {
        self.route(text).map(|s| s.extract(text))
    }

    /// Extract from text using all skills whose triggers match.
    pub fn extract_all_matching(&self, text: &str) -> HashMap<String, ExtractResult> {
        self.skills
            .values()
            .filter(|s| s.matches(text))
            .map(|s| (s.name.clone(), s.extract(text)))
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}