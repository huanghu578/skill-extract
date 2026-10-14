use std::collections::HashMap;
use std::sync::Arc;

/// A user-defined transformation applied to an extracted value.
///
/// Implement this trait and register the instance on a
/// [`PostProcessorRegistry`] to make it available to callers.
///
/// # Examples
///
/// ```
/// use skill_extract::PostProcessor;
///
/// struct Uppercase;
///
/// impl PostProcessor for Uppercase {
///     fn name(&self) -> &str { "uppercase" }
///     fn process(&self, value: &str) -> Option<String> {
///         Some(value.to_uppercase())
///     }
/// }
///
/// let p = Uppercase;
/// assert_eq!(p.process("abc"), Some("ABC".to_string()));
/// ```
pub trait PostProcessor: Send + Sync {
    /// The name used to look this processor up in a registry.
    fn name(&self) -> &str;

    /// Transform `value`. Return `None` to drop the value entirely.
    fn process(&self, value: &str) -> Option<String>;
}

/// A registry of named [`PostProcessor`]s.
///
/// This registry stores processors but does not automatically apply them.
/// Callers fetch a processor by name and invoke [`PostProcessor::process`].
#[derive(Default, Clone)]
pub struct PostProcessorRegistry {
    processors: HashMap<String, Arc<dyn PostProcessor>>,
}

impl PostProcessorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a processor. If a processor with the same name exists, it
    /// is replaced.
    pub fn register<P: PostProcessor + 'static>(&mut self, p: P) {
        self.processors.insert(p.name().to_string(), Arc::new(p));
    }

    /// Look up a processor by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn PostProcessor>> {
        self.processors.get(name).cloned()
    }

    /// List all registered processor names.
    pub fn names(&self) -> Vec<&str> {
        self.processors.keys().map(|s| s.as_str()).collect()
    }
}
