use assert_cmd::Command;
use predicates::prelude::*;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

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

#[test]
fn init_seeds_project_config_and_default_pack() {
    let workspace = temp_workspace("init");
    let project_config_dir = workspace.join("repo/.audit");
    let prompt_home = workspace.join("home");

    let mut cmd = Command::cargo_bin("ultraudit").unwrap();
    cmd.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success()
        .stdout(predicate::str::contains("init plan"));

    assert!(project_config_dir.join("config.toml").exists());
    assert!(project_config_dir.join("agents/codex.toml").exists());
    assert!(prompt_home
        .join("packs/ultraudit-default/versions/0.1.0/lenses/security/practices.md")
        .exists());
    assert!(prompt_home
        .join("packs/ultraudit-default/versions/0.1.0/optics/nice-practices/practices.md")
        .exists());
}

#[test]
fn run_executes_full_flow_with_shell_template_agent() {
    let workspace = temp_workspace("run");
    let repo = workspace.join("repo");
    let project_config_dir = repo.join(".audit");
    let prompt_home = workspace.join("home");
    let output_dir = workspace.join("runs");

    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        repo.join("src/lib.rs"),
        "pub fn sample() -> bool { true }\n",
    )
    .unwrap();

    let mut init = Command::cargo_bin("ultraudit").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();

    fs::write(
        project_config_dir.join("agents/test.toml"),
        r#"kind = "shell-template"
shell = "sh"
prompt_transport = "stdin"
timeout_seconds = 30
command = "mkdir -p $(dirname {{ report_path_sh }}) $(dirname {{ findings_path_sh }}) $(dirname {{ notes_path_sh }}); printf '# Agent Step\n\nok\n' > {{ report_path_sh }}; printf '[]\n' > {{ findings_path_sh }}; printf 'notes\n' > {{ notes_path_sh }}"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ultraudit").unwrap();
    cmd.arg("run")
        .arg("--repo")
        .arg(&repo)
        .arg("--config")
        .arg(project_config_dir.join("config.toml"))
        .arg("--prompt-home")
        .arg(&prompt_home)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--agent")
        .arg("test")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .assert()
        .success()
        .stdout(predicate::str::contains("audit complete"));

    let run_dir = newest_run_dir(&output_dir);
    assert!(run_dir.join("run.yaml").exists());
    assert!(run_dir.join("domain-map.yaml").exists());
    assert!(run_dir.join("raw/001-domain-discovery/prompt.md").exists());
    assert!(run_dir.join("reports/final-report.md").exists());
    assert!(run_dir
        .join("prompt-pack/lenses/architecture/practices.md")
        .exists());
}

fn temp_workspace(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("ultraudit-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn newest_run_dir(output_dir: &PathBuf) -> PathBuf {
    let mut runs = fs::read_dir(output_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    runs.sort();
    runs.pop().unwrap()
}
