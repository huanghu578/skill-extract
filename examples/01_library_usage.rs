//! Example 1: Single skill, direct extraction.
//!
//! Run with:
//!     cargo run --example 01_library_usage
//!
//! This shows the minimal usage: load one TOML, extract from text, print JSON.

use skill_extract::Skill;

fn main() {
    // Load a skill from a TOML file.
    let skill = match Skill::from_file("skills/dropper_task.toml") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load skill: {}", e);
            std::process::exit(1);
        }
    };

    // The input instruction.
    let text = "小dropper、小dropper，请从配置文件config.toml中找到 \
                suspension_type: THJ70+CTA85 的配置项，从 \
                输入文件:整体吊弦原始数据输入模板.xlsx 里读取原始输入数据，\
                计算后输出到 输出文件：计算结果.xlsx 中。";

    // Extract.
    let result = skill.extract(text);

    // Print as JSON.
    println!("{}", result.to_json());

    // Check completeness.
    if !result.is_complete() {
        eprintln!("Extraction incomplete.");
        eprintln!("  missing: {:?}", result.missing);
        eprintln!("  errors:  {:?}", result.errors);
        std::process::exit(2);
    }

    // Access values directly.
    if let Some(v) = result.get_first("suspension_type") {
        println!("suspension_type = {}", v);
    }
    if let Some(v) = result.get_first("input_file") {
        println!("input_file = {}", v);
    }
    if let Some(v) = result.get_first("output_file") {
        println!("output_file = {}", v);
    }
}
