use std::collections::HashMap;
use std::sync::Arc;

/// A user-registered post-processing step.
pub trait PostProcessor: Send + Sync {
    /// Name used in TOML config: `postprocess = ["my_processor"]`.
    fn name(&self) -> &str;
    /// Transform a value. Return `None` to drop it.
    fn process(&self, value: &str) -> Option<String>;
}

/// Registry of custom post-processors.
#[derive(Default, Clone)]
pub struct PostProcessorRegistry {
    processors: HashMap<String, Arc<dyn PostProcessor>>,
}

impl PostProcessorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P: PostProcessor + 'static>(&mut self, p: P) {
        self.processors.insert(p.name().to_string(), Arc::new(p));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn PostProcessor>> {
        self.processors.get(name).cloned()
    }

    pub fn names(&self) -> Vec<&str> {
        self.processors.keys().map(|s| s.as_str()).collect()
    }
}