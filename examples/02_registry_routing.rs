//! Example 2: Registry with trigger-word routing.
//!
//! Run with:
//!     cargo run --example 02_registry_routing
//!
//! This shows how to manage multiple skills and route instructions to the
//! right one based on trigger words.

use skill_extract::SkillRegistry;

fn main() {
    // Load all skills from the `skills/` directory.
    let registry = match SkillRegistry::load_dir("skills") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load skills: {}", e);
            std::process::exit(1);
        }
    };

    // List available skills.
    println!("Available skills:");
    for name in registry.list() {
        println!("  - {}", name);
    }
    println!();

    // A set of instructions, each targeting a different skill.
    let instructions = vec![
        "小dropper，请计算 suspension_type: THJ70+CTA85 \
         输入文件: a.xlsx 输出文件: b.xlsx",
        "小beam，请计算 beam_type: BX-100 \
         输入文件: beam_in.xlsx 输出文件: beam_out.xlsx",
        "请把数据保存到 工作文件夹: /tmp/work \
         处理结果写入 输出文件: result.csv",
        "这是一条无法匹配任何 skill 的指令。",
    ];

    for (i, text) in instructions.iter().enumerate() {
        println!("--- Instruction {} ---", i + 1);
        println!("Input: {}", text);

        match registry.extract_routed(text) {
            Some(result) if result.is_complete() => {
                println!("Routed to a skill. Values:");
                for (k, v) in &result.values {
                    println!("  {} = {:?}", k, v);
                }
            }
            Some(result) => {
                println!("Routed, but extraction incomplete:");
                println!("  missing: {:?}", result.missing);
                println!("  errors:  {:?}", result.errors);
            }
            None => {
                println!("No skill matched this instruction.");
            }
        }
        println!();
    }

    // You can also run ALL matching skills at once.
    let multi_match_text = "小dropper 小beam 工作文件夹: /tmp/a";
    let all = registry.extract_all_matching(multi_match_text);
    println!("--- All matching skills ---");
    println!("Input: {}", multi_match_text);
    for (name, result) in &all {
        println!("  skill [{}]: {} keys", name, result.values.len());
    }
}