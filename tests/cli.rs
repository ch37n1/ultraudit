use assert_cmd::Command;
use predicates::prelude::*;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn help_lists_core_commands() {
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: uat"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn version_comes_from_cargo_metadata() {
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn plan_prints_a_text_plan() {
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.args([
        "run",
        "--plan",
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
fn plan_can_emit_json() {
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.args(["--format", "json", "run", "--plan", "--pack", "production"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"pack\": \"production\""));
}

#[test]
fn plan_uses_ultraudit_path_for_prompt_home() {
    let workspace = temp_workspace("env");
    let prompt_home = workspace.join("for-test");
    let pack_source = prompt_home.join("packs/0.2.0");
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.env("ULTRAUDIT_PATH", &prompt_home)
        .args(["--format", "json", "run", "--plan"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "\"prompt_home\": \"{}\"",
            prompt_home.display()
        )))
        .stdout(predicate::str::contains(format!(
            "\"pack_source\": \"{}\"",
            pack_source.display()
        )));
}

#[test]
fn completions_can_be_generated() {
    let mut cmd = Command::cargo_bin("uat").unwrap();

    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_uat"));
}

#[test]
fn init_writes_project_config_without_seeding_pack() {
    let workspace = temp_workspace("init");
    let project_config_dir = workspace.join("repo/.audit");
    let prompt_home = workspace.join("home");

    let mut cmd = Command::cargo_bin("uat").unwrap();
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
    assert!(!prompt_home.join("packs/0.2.0").exists());

    let config = fs::read_to_string(project_config_dir.join("config.toml")).unwrap();
    assert!(config.contains("version = \"0.2.0\""));
    assert!(config.contains("packs/0.2.0"));
    assert!(!config.contains("name = \"ultraudit-default\""));
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

    let mut init = Command::cargo_bin("uat").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();
    install_test_pack(&prompt_home);

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

    let mut cmd = Command::cargo_bin("uat").unwrap();
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

    let lens_prompt = fs::read_to_string(run_dir.join("raw/002-core-architecture/prompt.md"))
        .expect("domain lens prompt should be preserved");
    assert!(lens_prompt.contains("# Architecture Reviewer Prompt Guidance"));
    assert!(lens_prompt.contains("# Architecture Practices"));
    assert!(lens_prompt.contains("# Evidence Model"));
    assert!(lens_prompt.contains("## Lens False-Positive Checks"));
    assert!(!run_dir
        .join("reports/optics/core.documentation-knowledge.md")
        .exists());
}

#[test]
fn run_prints_step_progress_without_agent_output_by_default() {
    let setup = noisy_shell_agent_setup("quiet-progress");

    let mut cmd = Command::cargo_bin("uat").unwrap();
    let output = cmd
        .arg("run")
        .arg("--repo")
        .arg(&setup.repo)
        .arg("--config")
        .arg(setup.project_config_dir.join("config.toml"))
        .arg("--prompt-home")
        .arg(&setup.prompt_home)
        .arg("--output-dir")
        .arg(&setup.output_dir)
        .arg("--agent")
        .arg("noisy")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr:\n{stderr}");
    assert!(stdout.contains("audit complete"));
    assert!(stderr.contains("start [1/7] domain discovery"));
    assert!(stderr.contains("done [7/7] final editorial verification"));
    assert!(!stderr.contains("agent stdout marker"));
    assert!(!stderr.contains("agent stderr marker"));

    let run_dir = newest_run_dir(&setup.output_dir);
    let raw_stdout = fs::read_to_string(run_dir.join("raw/001-domain-discovery/stdout.log"))
        .expect("agent stdout should still be logged");
    assert!(raw_stdout.contains("agent stdout marker"));
}

#[test]
fn verbose_run_streams_agent_output_to_stderr() {
    let setup = noisy_shell_agent_setup("verbose-output");

    let mut cmd = Command::cargo_bin("uat").unwrap();
    let output = cmd
        .arg("-v")
        .arg("run")
        .arg("--repo")
        .arg(&setup.repo)
        .arg("--config")
        .arg(setup.project_config_dir.join("config.toml"))
        .arg("--prompt-home")
        .arg(&setup.prompt_home)
        .arg("--output-dir")
        .arg(&setup.output_dir)
        .arg("--agent")
        .arg("noisy")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr:\n{stderr}");
    assert!(stdout.contains("audit complete"));
    assert!(stderr.contains("start [1/7] domain discovery"));
    assert!(stderr.contains("agent stdout marker"));
    assert!(stderr.contains("agent stderr marker"));
}

#[test]
fn json_run_keeps_stdout_machine_readable_and_hides_progress() {
    let setup = noisy_shell_agent_setup("json-output");

    let mut cmd = Command::cargo_bin("uat").unwrap();
    let output = cmd
        .arg("--format")
        .arg("json")
        .arg("run")
        .arg("--repo")
        .arg(&setup.repo)
        .arg("--config")
        .arg(setup.project_config_dir.join("config.toml"))
        .arg("--prompt-home")
        .arg(&setup.prompt_home)
        .arg("--output-dir")
        .arg(&setup.output_dir)
        .arg("--agent")
        .arg("noisy")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr:\n{stderr}");
    let summary: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(summary["dry_run"], false);
    assert!(!stdout.contains("start ["));
    assert!(!stdout.contains("agent stdout marker"));
    assert!(!stderr.contains("start ["));
    assert!(!stderr.contains("agent stdout marker"));
}

#[test]
fn codex_cli_agent_uses_current_codex_exec_flags() {
    let workspace = temp_workspace("codex-flags");
    let repo = workspace.join("repo");
    let project_config_dir = repo.join(".audit");
    let prompt_home = workspace.join("home");
    let output_dir = workspace.join("runs");
    let fake_codex = workspace.join("fake-codex");
    let recorded_args = workspace.join("codex-args.txt");

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
    fs::write(
        &fake_codex,
        "#!/bin/sh\nscript_dir=$(dirname \"$0\")\nprintf '%s\\n' \"$@\" > \"$script_dir/codex-args.txt\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_codex).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_codex, permissions).unwrap();

    let mut init = Command::cargo_bin("uat").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();
    install_test_pack(&prompt_home);

    fs::write(
        project_config_dir.join("agents/fake-codex.toml"),
        format!(
            r#"kind = "codex-cli"
binary = "{}"
mode = "exec"
prompt_transport = "stdin"
approval_policy = "never"
sandbox = "workspace-write"
timeout_seconds = 30
"#,
            fake_codex.display()
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("uat").unwrap();
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
        .arg("fake-codex")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .assert()
        .success();

    let args = fs::read_to_string(recorded_args).unwrap();
    assert_eq!(
        args,
        "--ask-for-approval\nnever\n--sandbox\nworkspace-write\nexec\n"
    );
}

#[test]
fn dry_run_executes_full_flow_without_real_agent() {
    let workspace = temp_workspace("dry-run");
    let repo = workspace.join("repo");
    let seed_config_dir = workspace.join("seed/.audit");
    let prompt_home = workspace.join("for-test");
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

    let mut init = Command::cargo_bin("uat").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&seed_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();
    install_test_pack(&prompt_home);

    let mut cmd = Command::cargo_bin("uat").unwrap();
    cmd.env("ULTRAUDIT_PATH", &prompt_home)
        .arg("run")
        .arg("--repo")
        .arg(&repo)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--dry-run")
        .arg("--agent")
        .arg("missing-agent")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .assert()
        .success()
        .stdout(predicate::str::contains("audit complete"))
        .stdout(predicate::str::contains("dry-run"));

    let run_dir = newest_run_dir(&output_dir);
    assert_readable_run_dir_name(&run_dir);
    assert!(run_dir.join("run.yaml").exists());
    assert!(run_dir.join("summary.yaml").exists());
    assert!(run_dir.join("domain-map.yaml").exists());
    assert!(run_dir.join("raw/001-domain-discovery/prompt.md").exists());
    assert!(run_dir.join("reports/final-report.md").exists());

    let invocation =
        fs::read_to_string(run_dir.join("raw/001-domain-discovery/invocation.yaml")).unwrap();
    assert!(invocation.contains("\"kind\": \"dry-run\""));
    assert!(invocation.contains("\"missing-agent\""));

    let exit = fs::read_to_string(run_dir.join("raw/001-domain-discovery/exit.json")).unwrap();
    assert!(exit.contains("\"success\": true"));
}

#[test]
fn run_fails_with_make_install_hint_when_pack_is_missing() {
    let workspace = temp_workspace("missing-pack");
    let repo = workspace.join("repo");
    let prompt_home = workspace.join("empty-home");
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

    let mut cmd = Command::cargo_bin("uat").unwrap();
    cmd.env("ULTRAUDIT_PATH", &prompt_home)
        .arg("run")
        .arg("--repo")
        .arg(&repo)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--dry-run")
        .arg("--lens")
        .arg("architecture")
        .arg("--domain")
        .arg("core")
        .assert()
        .failure()
        .stderr(predicate::str::contains("run `make install`"));
}

#[test]
fn default_flow_runs_project_optics_once_and_excludes_code_quality_from_cross_system() {
    let workspace = temp_workspace("default-flow");
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

    let mut init = Command::cargo_bin("uat").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();
    install_test_pack(&prompt_home);

    let mut cmd = Command::cargo_bin("uat").unwrap();
    cmd.env("ULTRAUDIT_PATH", &prompt_home)
        .arg("run")
        .arg("--repo")
        .arg(&repo)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--dry-run")
        .arg("--domain")
        .arg("core")
        .assert()
        .success();

    let run_dir = newest_run_dir(&output_dir);
    assert!(run_dir
        .join("reports/optics/documentation-knowledge.md")
        .exists());
    assert!(!run_dir
        .join("reports/optics/core.documentation-knowledge.md")
        .exists());

    let raw_steps = fs::read_dir(run_dir.join("raw"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        raw_steps
            .iter()
            .filter(|name| name.contains("documentation-knowledge"))
            .count(),
        1
    );

    let cross_system = fs::read_to_string(run_dir.join("raw/009-cross-system/prompt.md")).unwrap();
    assert!(cross_system.contains("architecture"));
    assert!(cross_system.contains("ml-ai"));
    assert!(!cross_system.contains("# Code Quality Practices"));
}

#[test]
fn makefile_install_target_installs_binary_pack_and_checks_codex() {
    let makefile = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
        .expect("Makefile should exist");

    assert!(makefile.contains("cargo build --release"));
    assert!(makefile.contains("target/release/uat"));
    assert!(makefile.contains("$(INSTALL_BIN)/uat"));
    assert!(makefile.contains("packs/$(PACK_VERSION)"));
    assert!(makefile.contains("command -v codex"));
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

struct ShellAgentSetup {
    repo: PathBuf,
    project_config_dir: PathBuf,
    prompt_home: PathBuf,
    output_dir: PathBuf,
}

fn noisy_shell_agent_setup(name: &str) -> ShellAgentSetup {
    let workspace = temp_workspace(name);
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

    let mut init = Command::cargo_bin("uat").unwrap();
    init.arg("init")
        .arg("--project-config-dir")
        .arg(&project_config_dir)
        .arg("--prompt-home")
        .arg(&prompt_home)
        .assert()
        .success();
    install_test_pack(&prompt_home);

    fs::write(
        project_config_dir.join("agents/noisy.toml"),
        r#"kind = "shell-template"
shell = "sh"
prompt_transport = "stdin"
timeout_seconds = 30
command = "printf 'agent %s marker\n' stdout; printf 'agent %s marker\n' stderr >&2; mkdir -p $(dirname {{ report_path_sh }}) $(dirname {{ findings_path_sh }}) $(dirname {{ notes_path_sh }}); printf '# Agent Step\n\nok\n' > {{ report_path_sh }}; printf '[]\n' > {{ findings_path_sh }}; printf 'notes\n' > {{ notes_path_sh }}"
"#,
    )
    .unwrap();

    ShellAgentSetup {
        repo,
        project_config_dir,
        prompt_home,
        output_dir,
    }
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

fn assert_readable_run_dir_name(run_dir: &Path) {
    let name = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("run directory should have a UTF-8 file name");

    assert!(
        is_datetime_run_dir_name(name),
        "unexpected run directory name: {name}"
    );
}

fn is_datetime_run_dir_name(name: &str) -> bool {
    if name.len() < 36 {
        return false;
    }

    let bytes = name.as_bytes();
    let expected_separators = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b'-'),
        (16, b'-'),
        (19, b'.'),
        (29, b'Z'),
        (30, b'-'),
        (31, b'r'),
        (32, b'u'),
        (33, b'n'),
        (34, b'-'),
    ];

    expected_separators
        .iter()
        .all(|(index, expected)| bytes.get(*index) == Some(expected))
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[17..19].iter().all(u8::is_ascii_digit)
        && bytes[20..29].iter().all(u8::is_ascii_digit)
        && bytes[35..].iter().all(u8::is_ascii_digit)
}

fn install_test_pack(prompt_home: &PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs/0.2.0");
    let target = prompt_home.join("packs/0.2.0");
    copy_dir_recursive(&source, &target);
}

fn copy_dir_recursive(source: &PathBuf, target: &PathBuf) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path);
        } else {
            fs::copy(source_path, target_path).unwrap();
        }
    }
}
