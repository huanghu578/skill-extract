//! Example 3: Custom post-processors.
//!
//! Run with:
//!     cargo run --example 03_custom_postprocessor
//!
//! This shows how to implement and register a custom `PostProcessor`.
//! Note: the TOML pipeline currently only supports built-in post-processors.
//! Custom ones are registered in the `PostProcessorRegistry` and can be
//! invoked programmatically.

use skill_extract::{PostProcessor, SkillRegistry};

/// A custom post-processor that extracts the file name from a path.
struct Basename;

impl PostProcessor for Basename {
    fn name(&self) -> &str {
        "basename"
    }

    fn process(&self, value: &str) -> Option<String> {
        std::path::Path::new(value)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
    }
}

/// A custom post-processor that uppercases the value.
struct Uppercase;

impl PostProcessor for Uppercase {
    fn name(&self) -> &str {
        "uppercase"
    }

    fn process(&self, value: &str) -> Option<String> {
        Some(value.to_uppercase())
    }
}

/// A custom post-processor that masks all but the last 4 characters.
struct MaskTail;

impl PostProcessor for MaskTail {
    fn name(&self) -> &str {
        "mask_tail"
    }

    fn process(&self, value: &str) -> Option<String> {
        let n = value.chars().count();
        if n <= 4 {
            return Some(value.to_string());
        }
        let masked: String = std::iter::repeat('*')
            .take(n - 4)
            .chain(value.chars().skip(n - 4))
            .collect();
        Some(masked)
    }
}

fn main() {
    // Create a registry (no skills loaded for this demo).
    let mut registry = SkillRegistry::new();

    // Register custom post-processors.
    registry.postprocessors.register(Basename);
    registry.postprocessors.register(Uppercase);
    registry.postprocessors.register(MaskTail);

    println!("Registered post-processors:");
    for name in registry.postprocessors.names() {
        println!("  - {}", name);
    }
    println!();

    // Apply each one to some sample values.
    let samples = vec![
        ("basename", "/home/user/project/result.csv"),
        ("uppercase", "thj70+cta85"),
        ("mask_tail", "13812345678"),
    ];

    for (name, input) in samples {
        if let Some(p) = registry.postprocessors.get(name) {
            match p.process(input) {
                Some(out) => println!("{}: {:?} -> {:?}", name, input, out),
                None => println!("{}: {:?} -> (dropped)", name, input),
            }
        } else {
            println!("{}: not registered", name);
        }
    }

    println!();
    println!("Note: to use a custom processor inside a TOML skill,");
    println!("the extraction pipeline must be extended to consult the registry.");
    println!("Currently only built-in post-processors are wired into Skill::extract.");
}