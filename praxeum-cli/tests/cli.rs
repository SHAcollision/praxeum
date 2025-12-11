use assert_cmd::Command;
use std::fs;

#[test]
fn validate_examples_passes() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("praxeum-cli"));
    cmd.args([
        "--file",
        "../praxeum-core/examples/exercises_basic.toml",
        "--validate-only",
    ])
    .assert()
    .success();
}

#[test]
fn invalid_file_exits_nonzero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("bad.toml");
    fs::write(&file_path, "not = [valid]").expect("write temp file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("praxeum-cli"));
    cmd.args(["--file", file_path.to_str().unwrap(), "--validate-only"])
        .assert()
        .failure();
}
