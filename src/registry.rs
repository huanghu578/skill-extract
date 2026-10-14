use crate::error::{Result, SkillError};
use crate::postprocess::PostProcessorRegistry;
use crate::skill::{ExtractResult, Skill};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manages a collection of [`Skill`]s.
///
/// Skills are indexed by name. The registry can load all `.toml` files from
/// a directory, route an instruction to the first matching skill by trigger
/// word, or run all matching skills at once.
///
/// # Examples
///
/// ```no_run
/// use skill_extract::SkillRegistry;
///
/// let registry = SkillRegistry::load_dir("skills").unwrap();
/// for name in registry.list() {
///     println!("{}", name);
/// }
/// ```
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    /// Registry of custom post-processors. See [`PostProcessorRegistry`].
    pub postprocessors: PostProcessorRegistry,
}

impl SkillRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            postprocessors: PostProcessorRegistry::new(),
        }
    }

    /// Load all `.toml` files from `dir` and register them.
    ///
    /// Files are loaded in directory order. The first error aborts the
    /// whole load; the registry is not partially populated.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError::Io`] if the directory cannot be read, or any
    /// error from [`Skill::from_file`] for individual skill files.
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

    /// Register a single skill, replacing any existing skill of the same name.
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Look up a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Return all skill names, sorted alphabetically.
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.skills.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Return the first skill whose triggers match `text`, if any.
    pub fn route(&self, text: &str) -> Option<&Skill> {
        self.skills.values().find(|s| s.matches(text))
    }

    /// Route `text` and extract with the matching skill.
    pub fn extract_routed(&self, text: &str) -> Option<ExtractResult> {
        self.route(text).map(|s| s.extract(text))
    }

    /// Extract with every skill whose triggers match `text`.
    ///
    /// Returns a map from skill name to its extraction result.
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
