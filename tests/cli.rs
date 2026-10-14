use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_skill-extract"))
}

#[test]
fn test_single_file_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let skill = tmp.path().join("s.toml");
    std::fs::write(
        &skill,
        r#"
name = "t"
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<value>\w+)'
required = true
"#,
    )
    .unwrap();

    let out = bin().arg(&skill).arg("x: hello").output().expect("run cli");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"x\""));
    assert!(stdout.contains("hello"));
}

#[test]
fn test_missing_required_exits_2() {
    let tmp = tempfile::tempdir().unwrap();
    let skill = tmp.path().join("s.toml");
    std::fs::write(
        &skill,
        r#"
name = "t"
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<value>\w+)'
required = true
"#,
    )
    .unwrap();

    let out = bin()
        .arg(&skill)
        .arg("no match here")
        .output()
        .expect("run cli");

    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn test_dir_list_mode() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("a.toml"),
        r#"
name = "alpha"
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<value>\w+)'
"#,
    )
    .unwrap();

    let out = bin()
        .arg("--dir")
        .arg(tmp.path())
        .arg("--list")
        .output()
        .expect("run cli");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alpha"));
}

#[test]
fn test_dir_route_mode() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("a.toml"),
        r#"
name = "alpha"
triggers = ["alpha"]
[[extractors]]
key = "x"
pattern = 'x[:：]\s*(?P<value>\w+)'
required = true
"#,
    )
    .unwrap();

    let out = bin()
        .arg("--dir")
        .arg(tmp.path())
        .arg("--route")
        .arg("please run alpha with x: ok")
        .output()
        .expect("run cli");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ok"));
}
