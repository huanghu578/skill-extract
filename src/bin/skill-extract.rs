//! Command-line entry point for `skill-extract`.
//!
//! Supports four modes:
//!
//! ```text
//! skill-extract <skill.toml> <text>
//! skill-extract --dir <skills_dir> --route <text>
//! skill-extract --dir <skills_dir> --list
//! skill-extract <skill.toml> --stdin
//! ```
//!
//! Exit codes:
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success; all required keys present |
//! | 1 | Configuration or I/O error |
//! | 2 | Extraction incomplete |
//! | 3 | No skill matched (route mode only) |

use skill_extract::{Skill, SkillRegistry};
use std::env;
use std::process;

fn usage() {
    eprintln!("Usage:");
    eprintln!("  skill-extract <skill.toml> <text>");
    eprintln!("  skill-extract --dir <skills_dir> --route <text>");
    eprintln!("  skill-extract --dir <skills_dir> --list");
    eprintln!("  skill-extract <skill.toml> --stdin");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        process::exit(1);
    }

    // --dir mode: use the registry
    if args[1] == "--dir" {
        if args.len() < 3 {
            usage();
            process::exit(1);
        }
        let dir = &args[2];
        let registry = match SkillRegistry::load_dir(dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load skills from [{}]: {}", dir, e);
                process::exit(1);
            }
        };

        if args.len() >= 4 && args[3] == "--list" {
            for name in registry.list() {
                println!("{}", name);
            }
            return;
        }

        if args.len() >= 5 && args[3] == "--route" {
            let text = &args[4];
            match registry.extract_routed(text) {
                Some(result) => {
                    println!("{}", result.to_json());
                    if !result.is_complete() {
                        process::exit(2);
                    }
                }
                None => {
                    eprintln!("No skill matched the input.");
                    process::exit(3);
                }
            }
            return;
        }

        usage();
        process::exit(1);
    }

    // Single-file mode
    if args.len() < 3 {
        usage();
        process::exit(1);
    }

    let skill_path = &args[1];
    let skill = match Skill::from_file(skill_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load skill [{}]: {}", skill_path, e);
            process::exit(1);
        }
    };

    let text = if args[2] == "--stdin" {
        use std::io::Read;
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Failed to read stdin: {}", e);
            process::exit(1);
        }
        buf
    } else {
        args[2].clone()
    };

    let result = skill.extract(&text);
    println!("{}", result.to_json());

    if !result.is_complete() {
        process::exit(2);
    }
}
