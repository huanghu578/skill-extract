use skill_extract::{PostProcessor, SkillRegistry};
use std::fs;

const SKILL_A: &str = r#"
name = "skill_a"
triggers = ["alpha"]
[[extractors]]
key = "alpha_value"
pattern = 'alpha[:：]\s*(?P<value>\w+)'
required = true
"#;

const SKILL_B: &str = r#"
name = "skill_b"
triggers = ["beta"]
[[extractors]]
key = "beta_value"
pattern = 'beta[:：]\s*(?P<value>\w+)'
required = true
"#;

fn write_skills(dir: &std::path::Path) {
    fs::write(dir.join("a.toml"), SKILL_A).unwrap();
    fs::write(dir.join("b.toml"), SKILL_B).unwrap();
}

#[test]
fn test_load_dir_and_list() {
    let tmp = tempfile::tempdir().unwrap();
    write_skills(tmp.path());

    let registry = SkillRegistry::load_dir(tmp.path()).unwrap();
    let mut names = registry.list();
    names.sort();
    assert_eq!(names, vec!["skill_a", "skill_b"]);
}

#[test]
fn test_route_by_trigger() {
    let tmp = tempfile::tempdir().unwrap();
    write_skills(tmp.path());

    let registry = SkillRegistry::load_dir(tmp.path()).unwrap();

    let skill = registry.route("please run alpha now").unwrap();
    assert_eq!(skill.name, "skill_a");

    let skill = registry.route("please run beta now").unwrap();
    assert_eq!(skill.name, "skill_b");

    assert!(registry.route("no trigger here").is_none());
}

#[test]
fn test_extract_routed() {
    let tmp = tempfile::tempdir().unwrap();
    write_skills(tmp.path());

    let registry = SkillRegistry::load_dir(tmp.path()).unwrap();
    let result = registry.extract_routed("alpha: hello").unwrap();
    assert_eq!(result.values["alpha_value"], vec!["hello"]);
}

#[test]
fn test_extract_all_matching() {
    let tmp = tempfile::tempdir().unwrap();
    let both = r#"
name = "both"
triggers = ["alpha", "beta"]
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<value>\w+)'
"#;
    fs::write(tmp.path().join("both.toml"), both).unwrap();
    write_skills(tmp.path());

    let registry = SkillRegistry::load_dir(tmp.path()).unwrap();
    let results = registry.extract_all_matching("alpha and beta");
    assert_eq!(results.len(), 3);
}

struct Uppercase;

impl PostProcessor for Uppercase {
    fn name(&self) -> &str {
        "uppercase"
    }
    fn process(&self, value: &str) -> Option<String> {
        Some(value.to_uppercase())
    }
}

#[test]
fn test_register_custom_postprocessor() {
    let mut registry = SkillRegistry::new();
    registry.postprocessors.register(Uppercase);
    assert!(registry.postprocessors.get("uppercase").is_some());
    assert!(registry.postprocessors.names().contains(&"uppercase"));

    let p = registry.postprocessors.get("uppercase").unwrap();
    assert_eq!(p.process("hello"), Some("HELLO".to_string()));
}

#[test]
fn test_load_nonexistent_dir_fails() {
    let result = SkillRegistry::load_dir("/nonexistent/path/xyz");
    assert!(result.is_err());
}

#[test]
fn test_registry_default_is_empty() {
    let registry = SkillRegistry::default();
    assert!(registry.list().is_empty());
}