use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use wait_timeout::ChildExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::{
    config::{AgentConfig, AgentKind, PromptTransport},
    model::{AgentExit, AgentInvocationManifest, AgentStepRecord},
    pack::render_template,
    util::{command_display, shell_escape, write_json_yaml, write_text},
};

#[derive(Debug, Clone)]
pub struct AgentRunRequest {
    pub step_id: String,
    pub role: String,
    pub cwd: PathBuf,
    pub prompt: String,
    pub raw_dir: PathBuf,
    pub report_path: PathBuf,
    pub findings_path: PathBuf,
    pub notes_path: PathBuf,
    pub allow_failure: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AgentRunOptions {
    pub live_output: bool,
}

pub fn run_agent(
    config: &AgentConfig,
    request: &AgentRunRequest,
    options: AgentRunOptions,
) -> Result<AgentStepRecord> {
    std::fs::create_dir_all(&request.raw_dir)
        .with_context(|| format!("create {}", request.raw_dir.display()))?;

    let prompt_path = request.raw_dir.join("prompt.md");
    let stdout_path = request.raw_dir.join("stdout.log");
    let stderr_path = request.raw_dir.join("stderr.log");
    let invocation_path = request.raw_dir.join("invocation.yaml");
    let command_path = request.raw_dir.join("command.txt");
    let exit_path = request.raw_dir.join("exit.json");

    write_text(&prompt_path, &request.prompt)?;

    let built = build_invocation(config, request, &prompt_path, &stdout_path, &stderr_path)?;
    write_text(&command_path, command_display(&built.program, &built.args))?;
    write_json_yaml(&invocation_path, &built)?;

    let started = Instant::now();
    let stdout_file =
        File::create(&stdout_path).with_context(|| format!("create {}", stdout_path.display()))?;
    let stderr_file =
        File::create(&stderr_path).with_context(|| format!("create {}", stderr_path.display()))?;

    let mut command = Command::new(&built.program);
    command.args(&built.args).current_dir(&built.cwd);
    configure_process_group(&mut command);

    let output_files = if options.live_output {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        Some((stdout_file, stderr_file))
    } else {
        command
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file));
        None
    };

    for (key, value) in &config.env {
        command.env(key, value);
    }

    if matches!(config.prompt_transport, PromptTransport::Stdin) {
        command.stdin(Stdio::piped());
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            drop(output_files);
            let exit = AgentExit {
                success: false,
                exit_code: None,
                timed_out: false,
                duration_ms: started.elapsed().as_millis(),
                error: Some(error.to_string()),
            };
            write_text(&stderr_path, format!("failed to spawn agent: {error}\n"))?;
            write_json_yaml(&exit_path, &exit)?;
            let record = AgentStepRecord {
                invocation: built,
                exit,
            };
            return if request.allow_failure {
                Ok(record)
            } else {
                Err(anyhow!("failed to start agent `{}`: {error}", config.name))
            };
        }
    };

    let output_threads = if let Some((stdout_file, stderr_file)) = output_files {
        let live_lock = Arc::new(Mutex::new(()));
        let mut threads = Vec::new();
        if let Some(stdout) = child.stdout.take() {
            threads.push(spawn_output_tee(
                stdout,
                stdout_file,
                Arc::clone(&live_lock),
            ));
        }
        if let Some(stderr) = child.stderr.take() {
            threads.push(spawn_output_tee(stderr, stderr_file, live_lock));
        }
        threads
    } else {
        Vec::new()
    };

    if matches!(config.prompt_transport, PromptTransport::Stdin) {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(request.prompt.as_bytes())
                .with_context(|| format!("write stdin for step {}", request.step_id))?;
        }
    }

    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let status = if config.timeout_seconds == 0 {
        Some(child.wait().with_context(|| {
            format!("wait for agent `{}` step {}", config.name, request.step_id)
        })?)
    } else {
        child
            .wait_timeout(timeout)
            .with_context(|| format!("wait with timeout for step {}", request.step_id))?
    };

    let (timed_out, exit_code, success, error) = if let Some(status) = status {
        (
            false,
            status.code(),
            status.success(),
            (!status.success()).then(|| format!("agent exited with status {status}")),
        )
    } else {
        terminate_child_process(&mut child);
        (
            true,
            None,
            false,
            Some(format!(
                "agent timed out after {} seconds",
                config.timeout_seconds
            )),
        )
    };
    let output_result = join_output_threads(output_threads);

    let exit = AgentExit {
        success,
        exit_code,
        timed_out,
        duration_ms: started.elapsed().as_millis(),
        error,
    };
    write_json_yaml(&exit_path, &exit)?;
    output_result?;

    let record = AgentStepRecord {
        invocation: built,
        exit,
    };

    if record.exit.success || request.allow_failure {
        Ok(record)
    } else {
        Err(anyhow!(
            "agent `{}` failed during step `{}`; artifacts are in {}",
            config.name,
            request.step_id,
            request.raw_dir.display()
        ))
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

fn terminate_child_process(child: &mut Child) {
    #[cfg(unix)]
    {
        let process_group = format!("-{}", child.id());
        let _ = Command::new("kill")
            .args(["-TERM", &process_group])
            .status();
        thread::sleep(Duration::from_millis(100));
        if matches!(child.try_wait(), Ok(None)) {
            let _ = Command::new("kill")
                .args(["-KILL", &process_group])
                .status();
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    let _ = child.wait();
}

fn spawn_output_tee<R>(
    mut reader: R,
    mut file: File,
    live_lock: Arc<Mutex<()>>,
) -> JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            let read = reader.read(&mut buffer).context("read agent output")?;
            if read == 0 {
                break;
            }

            let chunk = &buffer[..read];
            file.write_all(chunk).context("write agent output log")?;
            write_live_agent_output(chunk, &live_lock);
        }
        file.flush().context("flush agent output log")
    })
}

fn join_output_threads(threads: Vec<JoinHandle<Result<()>>>) -> Result<()> {
    for thread in threads {
        thread
            .join()
            .map_err(|_| anyhow!("agent output tee thread panicked"))??;
    }
    Ok(())
}

fn write_live_agent_output(chunk: &[u8], live_lock: &Arc<Mutex<()>>) {
    let Ok(_guard) = live_lock.lock() else {
        return;
    };
    let mut stderr = io::stderr().lock();
    let _ = stderr.write_all(chunk);
    let _ = stderr.flush();
}

pub fn run_agent_dry_run(agent_name: &str, request: &AgentRunRequest) -> Result<AgentStepRecord> {
    std::fs::create_dir_all(&request.raw_dir)
        .with_context(|| format!("create {}", request.raw_dir.display()))?;

    let prompt_path = request.raw_dir.join("prompt.md");
    let stdout_path = request.raw_dir.join("stdout.log");
    let stderr_path = request.raw_dir.join("stderr.log");
    let invocation_path = request.raw_dir.join("invocation.yaml");
    let command_path = request.raw_dir.join("command.txt");
    let exit_path = request.raw_dir.join("exit.json");

    write_text(&prompt_path, &request.prompt)?;

    let invocation = AgentInvocationManifest {
        step_id: request.step_id.clone(),
        role: request.role.clone(),
        kind: "dry-run".to_owned(),
        program: PathBuf::from("ultraudit-dry-run"),
        args: vec![
            "--agent".to_owned(),
            agent_name.to_owned(),
            "--step".to_owned(),
            request.step_id.clone(),
        ],
        cwd: request.cwd.clone(),
        prompt_transport: "none".to_owned(),
        timeout_seconds: 0,
        prompt_path,
        stdout_path: stdout_path.clone(),
        stderr_path: stderr_path.clone(),
        report_path: request.report_path.clone(),
        findings_path: request.findings_path.clone(),
        notes_path: request.notes_path.clone(),
    };

    write_text(
        &command_path,
        command_display(&invocation.program, &invocation.args),
    )?;
    write_json_yaml(&invocation_path, &invocation)?;
    write_text(
        &stdout_path,
        format!(
            "dry-run: skipped agent `{agent_name}` for step `{}`\n",
            request.step_id
        ),
    )?;
    write_text(&stderr_path, "")?;
    write_text(
        &request.notes_path,
        format!(
            "# Reviewer Notes\n\nDry-run mode skipped real agent execution for step `{}`.\n",
            request.step_id
        ),
    )?;

    let exit = AgentExit {
        success: true,
        exit_code: Some(0),
        timed_out: false,
        duration_ms: 0,
        error: None,
    };
    write_json_yaml(&exit_path, &exit)?;

    Ok(AgentStepRecord { invocation, exit })
}

fn build_invocation(
    config: &AgentConfig,
    request: &AgentRunRequest,
    prompt_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<AgentInvocationManifest> {
    let (program, args) = match config.kind {
        AgentKind::CodexCli => build_codex_invocation(config),
        AgentKind::ShellTemplate => build_shell_invocation(config, request, prompt_path)?,
    };

    Ok(AgentInvocationManifest {
        step_id: request.step_id.clone(),
        role: request.role.clone(),
        kind: config.kind_name().to_owned(),
        program,
        args,
        cwd: request.cwd.clone(),
        prompt_transport: config.prompt_transport_name().to_owned(),
        timeout_seconds: config.timeout_seconds,
        prompt_path: prompt_path.to_path_buf(),
        stdout_path: stdout_path.to_path_buf(),
        stderr_path: stderr_path.to_path_buf(),
        report_path: request.report_path.clone(),
        findings_path: request.findings_path.clone(),
        notes_path: request.notes_path.clone(),
    })
}

fn build_codex_invocation(config: &AgentConfig) -> (PathBuf, Vec<String>) {
    let mut args = Vec::new();
    if !config.approval_policy.is_empty() {
        args.push("--ask-for-approval".to_owned());
        args.push(config.approval_policy.clone());
    }
    if !config.sandbox.is_empty() {
        args.push("--sandbox".to_owned());
        args.push(config.sandbox.clone());
    }
    if let Some(model) = &config.model {
        args.push("--model".to_owned());
        args.push(model.clone());
    }
    if !config.mode.is_empty() {
        args.push(config.mode.clone());
    }
    if config.ignore_user_config {
        args.push("--ignore-user-config".to_owned());
    }

    (config.binary.clone(), args)
}

fn build_shell_invocation(
    config: &AgentConfig,
    request: &AgentRunRequest,
    prompt_path: &Path,
) -> Result<(PathBuf, Vec<String>)> {
    let Some(command) = config.command.as_deref() else {
        return Err(anyhow!(
            "shell-template agent `{}` has no command",
            config.name
        ));
    };

    let mut values = BTreeMap::new();
    values.insert("step_id", request.step_id.clone());
    values.insert("cwd", request.cwd.display().to_string());
    values.insert("cwd_sh", shell_escape(&request.cwd.display().to_string()));
    values.insert("prompt_path", prompt_path.display().to_string());
    values.insert(
        "prompt_path_sh",
        shell_escape(&prompt_path.display().to_string()),
    );
    values.insert("output_dir", request.raw_dir.display().to_string());
    values.insert(
        "output_dir_sh",
        shell_escape(&request.raw_dir.display().to_string()),
    );
    values.insert("report_path", request.report_path.display().to_string());
    values.insert(
        "report_path_sh",
        shell_escape(&request.report_path.display().to_string()),
    );
    values.insert("findings_path", request.findings_path.display().to_string());
    values.insert(
        "findings_path_sh",
        shell_escape(&request.findings_path.display().to_string()),
    );
    values.insert("notes_path", request.notes_path.display().to_string());
    values.insert(
        "notes_path_sh",
        shell_escape(&request.notes_path.display().to_string()),
    );

    let command = render_template(command, &values);
    Ok((PathBuf::from(&config.shell), vec!["-c".to_owned(), command]))
}
