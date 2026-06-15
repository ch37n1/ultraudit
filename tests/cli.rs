use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_core_commands() {
    let mut cmd = Command::cargo_bin("ultraudit").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn version_comes_from_cargo_metadata() {
    let mut cmd = Command::cargo_bin("ultraudit").unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn dry_run_prints_a_text_plan() {
    let mut cmd = Command::cargo_bin("ultraudit").unwrap();

    cmd.args([
        "run",
        "--dry-run",
        "--pack",
        "full",
        "--lens",
        "security",
        "--optic",
        "nice-practices",
        "--domain",
        "auth",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("audit plan"))
    .stdout(predicate::str::contains("security"))
    .stdout(predicate::str::contains("nice-practices"))
    .stdout(predicate::str::contains("auth"));
}

#[test]
fn dry_run_can_emit_json() {
    let mut cmd = Command::cargo_bin("ultraudit").unwrap();

    cmd.args([
        "--format",
        "json",
        "run",
        "--dry-run",
        "--pack",
        "production",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"pack\": \"production\""));
}

#[test]
fn completions_can_be_generated() {
    let mut cmd = Command::cargo_bin("ultraudit").unwrap();

    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_ultraudit"));
}
