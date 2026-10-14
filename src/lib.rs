//! # skill-extract
//!
//! Configuration-driven key-value extraction from natural language instructions.
//!
//! A **skill** is a TOML file that declares what to extract. Each **extractor**
//! is a regex with a fixed `(?P<value>...)` capture group. Keys can be any
//! Unicode string — Chinese, Japanese, emoji, anything.
//!
//! ## Quick start
//!
//! ```
//! use skill_extract::Skill;
//!
//! let toml = r#"
//! name = "demo"
//! [[extractors]]
//! key = "path"
//! pattern = 'path[:：]\s*(?P<value>\S+)'
//! required = true
//! "#;
//!
//! let skill = Skill::from_toml(toml).unwrap();
//! let result = skill.extract("path: /tmp/data");
//! assert_eq!(result.get_first("path"), Some("/tmp/data"));
//! assert!(result.is_complete());
//! ```
//!
//! ## Architecture
//!
//! - [`Skill`] — a compiled TOML configuration. Compiles regexes once, then
//!   runs them against text.
//! - [`SkillRegistry`] — manages multiple skills, loads them from a directory,
//!   routes instructions by trigger word.
//! - [`ExtractResult`] — the output: `values`, `missing`, `errors`.
//! - [`PostProcessor`] — trait for user-defined value transformations.
//! - [`SkillError`] — structured error type with context on every variant.
//!
//! ## Design principles
//!
//! - **Config-driven**: extraction rules live in TOML, not code.
//! - **Unicode-native**: keys are `String`, patterns use Unicode-aware regex.
//! - **Compile-time strict**: all regex compilation happens in
//!   [`Skill::from_config`]. Once a `Skill` exists, `extract` cannot fail for
//!   configuration reasons.
//! - **Structured output**: JSON with separate `missing` and `errors` fields.
//!
//! ## Feature overview
//!
//! | Capability | Entry point |
//! |------------|-------------|
//! | Load a single skill | [`Skill::from_file`] |
//! | Extract key-value pairs | [`Skill::extract`] |
//! | Load many skills | [`SkillRegistry::load_dir`] |
//! | Route by trigger word | [`SkillRegistry::route`] |
//! | Validate output types | `[output_schema]` in TOML |
//! | Custom post-processing | [`PostProcessor`] trait |
//!
//! ## CLI
//!
//! A binary is also shipped:
//!
//! ```text
//! skill-extract <skill.toml> <text>
//! skill-extract --dir <skills_dir> --route <text>
//! skill-extract --dir <skills_dir> --list
//! ```
//!
//! ## License
//!
//! MIT OR Apache-2.0

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
