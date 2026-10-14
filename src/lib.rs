//! skill-extract: configuration-driven key-value extraction from text.
//!
//! This crate provides a `Skill` that compiles a TOML configuration into a
//! set of regex-based extractors, plus a `SkillRegistry` for managing multiple
//! skills and routing by trigger word.

pub mod config;
pub mod error;
pub mod postprocess;
pub mod registry;
pub mod schema;
pub mod skill;

pub use config::{ExtractorConfig, FieldSpec, OutputSchema, PostProcess, SkillConfig};
pub use error::{Result, SkillError};
pub use postprocess::{PostProcessor, PostProcessorRegistry};
pub use registry::SkillRegistry;
pub use skill::{ExtractResult, Skill};