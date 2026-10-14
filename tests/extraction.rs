use skill_extract::Skill;

const TOML: &str = r#"
name = "extract_paths"
description = "test skill"
triggers = ["工作文件夹"]

[[extractors]]
key = "工作文件夹"
key_en = "working_dir"
pattern = '工作文件夹[:：]\s*(?P<value>[^\s，,。；;、]+)'
required = true
multi = false
postprocess = ["trim", "strip_quotes", "normalize_path"]

[[extractors]]
key = "出力ファイル"
key_en = "output_file"
pattern = '出力ファイル[:：]\s*(?P<value>[^\s，,。；;、]+)'
required = false
multi = false
postprocess = ["trim", "strip_quotes", "normalize_path"]
"#;

#[test]
fn test_basic_extract() {
    let skill = Skill::from_toml(TOML).unwrap();
    let text = "工作文件夹: /home/user/project 出力ファイル: result.csv";
    let result = skill.extract(text);

    assert_eq!(result.values["工作文件夹"], vec!["/home/user/project"]);
    assert_eq!(result.values["working_dir"], vec!["/home/user/project"]);
    assert_eq!(result.values["出力ファイル"], vec!["result.csv"]);
    assert_eq!(result.values["output_file"], vec!["result.csv"]);
    assert!(result.missing.is_empty());
    assert!(result.is_complete());
}

#[test]
fn test_unicode_keys_supported() {
    let skill = Skill::from_toml(TOML).unwrap();
    let text = "工作文件夹： /tmp/a 出力ファイル： out.csv";
    let result = skill.extract(text);
    assert_eq!(result.values["工作文件夹"], vec!["/tmp/a"]);
    assert_eq!(result.values["出力ファイル"], vec!["out.csv"]);
}

#[test]
fn test_backslash_normalize() {
    let skill = Skill::from_toml(TOML).unwrap();
    let text = "工作文件夹: D:\\data\\output";
    let result = skill.extract(text);
    assert_eq!(result.values["工作文件夹"], vec!["D:/data/output"]);
}

#[test]
fn test_missing_required() {
    let skill = Skill::from_toml(TOML).unwrap();
    let text = "nothing here";
    let result = skill.extract(text);
    assert!(!result.is_complete());
    assert_eq!(result.missing, vec!["工作文件夹"]);
    assert!(!result.missing.contains(&"出力ファイル".to_string()));
}

#[test]
fn test_strip_quotes() {
    let skill = Skill::from_toml(TOML).unwrap();
    let text = r#"工作文件夹: "/tmp/a""#;
    let result = skill.extract(text);
    assert_eq!(result.values["工作文件夹"], vec!["/tmp/a"]);
}

#[test]
fn test_multi_matches() {
    let toml = r#"
name = "multi"
[[extractors]]
key = "id"
pattern = 'id[:：]\s*(?P<value>\d+)'
multi = true
"#;
    let skill = Skill::from_toml(toml).unwrap();
    let result = skill.extract("id: 1 id: 2 id: 3");
    assert_eq!(result.values["id"], vec!["1", "2", "3"]);
}

#[test]
fn test_single_match() {
    let toml = r#"
name = "single"
[[extractors]]
key = "id"
pattern = 'id[:：]\s*(?P<value>\d+)'
multi = false
"#;
    let skill = Skill::from_toml(toml).unwrap();
    let result = skill.extract("id: 1 id: 2 id: 3");
    assert_eq!(result.values["id"], vec!["1"]);
}

#[test]
fn test_schema_validation_pass() {
    let toml = r#"
name = "schema_ok"
[[extractors]]
key = "count"
pattern = 'count[:：]\s*(?P<value>\d+)'
[output_schema]
count = { type = "int", required = true }
"#;
    let skill = Skill::from_toml(toml).unwrap();
    let result = skill.extract("count: 42");
    assert!(result.errors.is_empty());
}

#[test]
fn test_schema_validation_type_mismatch() {
    let toml = r#"
name = "schema_fail"
[[extractors]]
key = "count"
pattern = 'count[:：]\s*(?P<value>\w+)'
[output_schema]
count = { type = "int", required = true }
"#;
    let skill = Skill::from_toml(toml).unwrap();
    let result = skill.extract("count: abc");
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("type mismatch"));
}

#[test]
fn test_invalid_pattern_rejected() {
    let toml = r#"
name = "bad"
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<not_value>\w+)'
"#;
    let err = Skill::from_toml(toml).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("(?P<value>"));
}

#[test]
fn test_trigger_matching() {
    let skill = Skill::from_toml(TOML).unwrap();
    assert!(skill.matches("请用工作文件夹处理"));
    assert!(!skill.matches("unrelated text"));
}