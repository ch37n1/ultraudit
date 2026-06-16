use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        mpsc::{self, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use terminal_size::{terminal_size, Width};

use crate::{
    cli::{Lens, Optic, RunArgs},
    config::{
        find_project_config, load_effective_agent_config, load_project_config, resolve_agent_model,
        AgentConfig,
    },
    intake::{
        collect_repository_context, domain_map_markdown, domain_markdown, infer_domains,
        repository_markdown, write_repository_artifacts,
    },
    model::{
        AgentStepRecord, AuditPlan, Domain, DomainMap, InitPlan, PromptPackManifest, RunManifest,
        RunSummary,
    },
    pack::{
        legacy_pack_root, load_optional_pack_policy, pack_root, read_pack_policy, read_pack_text,
        render_template, snapshot_pack, validate_pack_files, Pack, PackPolicy,
    },
    runner::{run_agent, run_agent_dry_run, AgentRunOptions, AgentRunRequest},
    util::{
        copy_dir_recursive, now_run_id, read_optional, resolve_path, sanitize_id, truncate_chars,
        write_json_yaml, write_text, write_text_if_absent,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct AuditExecutionOptions {
    pub progress: ProgressDisplay,
    pub verbose_agent_output: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProgressDisplay {
    Hidden,
    Lines,
    Spinner,
}

const DEFAULT_PROMPT_MAX_CHARS: usize = 180_000;

pub fn build_audit_plan(args: &RunArgs) -> Result<AuditPlan> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let repository = resolve_path(&args.repository, &cwd);
    let config = find_project_config(args.config.as_deref(), &repository);
    let project_config = load_project_config(config.as_deref())?;
    let prompt_home = resolve_path(&args.prompt_home, &cwd);
    let output_dir = resolve_output_dir(args, &project_config.output_dir, &repository, &cwd);
    let pack_name = selected_pack_name(args, &project_config.prompt_pack_name);
    let pack_version = selected_pack_version(args, &project_config.prompt_pack_version);
    let pack_source = project_config
        .prompt_pack_source
        .clone()
        .unwrap_or_else(|| pack_root(&prompt_home, &pack_name, &pack_version));
    let pack_policy = load_optional_pack_policy(&pack_source)?.unwrap_or_else(PackPolicy::builtin);
    let lenses = selected_lenses(args, &pack_policy);
    let optics = selected_optics(args, &project_config.disabled_optics, &pack_policy);
    let previous_runs = selected_previous_runs(args, &output_dir)?;

    let agent = selected_agent(args, &project_config.default_agent);
    let model = resolve_agent_model(
        &agent,
        config.as_deref(),
        &repository,
        args.dry_run || args.plan,
    )?;

    Ok(AuditPlan {
        repository,
        output_dir,
        config,
        prompt_home,
        pack_name,
        pack_version,
        pack_source,
        pack: args.pack.as_str().to_owned(),
        agent,
        model,
        lenses: lenses.iter().map(|lens| lens.as_str().to_owned()).collect(),
        optics: optics
            .iter()
            .map(|optic| optic.as_str().to_owned())
            .collect(),
        domains: args.domains.clone(),
        previous_runs,
        dry_run: args.dry_run,
        allow_agent_failures: args.allow_agent_failures,
        jobs: args.jobs,
        retries: args.retries,
    })
}

pub fn init_project(args: &crate::cli::InitArgs) -> Result<InitPlan> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let project_config_dir = resolve_path(&args.project_config_dir, &cwd);
    let prompt_home = resolve_path(&args.prompt_home, &cwd);

    if args.dry_run {
        return Ok(InitPlan {
            project_config_dir,
            prompt_home,
            pack_name: args.pack_name.clone(),
            pack_version: args.pack_version.clone(),
            force: args.force,
            dry_run: args.dry_run,
        });
    }

    fs::create_dir_all(project_config_dir.join("agents"))
        .with_context(|| format!("create {}", project_config_dir.join("agents").display()))?;

    write_text_if_absent(
        &project_config_dir.join("config.toml"),
        project_config_template(
            &args.pack_version,
            &pack_root(&prompt_home, &args.pack_name, &args.pack_version),
        ),
        args.force,
    )?;
    write_text_if_absent(
        &project_config_dir.join("agents/codex.toml"),
        codex_agent_template(),
        args.force,
    )?;
    write_text_if_absent(
        &project_config_dir.join("agents/custom-shell.toml"),
        custom_shell_agent_template(),
        args.force,
    )?;

    Ok(InitPlan {
        project_config_dir,
        prompt_home,
        pack_name: args.pack_name.clone(),
        pack_version: args.pack_version.clone(),
        force: args.force,
        dry_run: args.dry_run,
    })
}

pub fn execute_audit(args: &RunArgs, options: AuditExecutionOptions) -> Result<RunSummary> {
    let plan = build_audit_plan(args)?;
    if plan.lenses.is_empty() && plan.optics.is_empty() {
        bail!("no lenses or optics selected");
    }

    let project_config = load_project_config(plan.config.as_deref())?;
    let resolved_pack = resolve_pack(&plan, project_config.prompt_pack_source.as_deref())?;
    let pack = resolved_pack.pack;
    let pack_policy = resolved_pack.policy;

    fs::create_dir_all(&plan.output_dir)
        .with_context(|| format!("create {}", plan.output_dir.display()))?;
    if let Some(retention_days) = project_config.artifact_retention_days {
        prune_old_runs(&plan.output_dir, retention_days)?;
    }
    let run_id = now_run_id();
    let run_dir = plan.output_dir.join(&run_id);
    fs::create_dir_all(&run_dir).with_context(|| format!("create {}", run_dir.display()))?;

    let snapshot = snapshot_pack(&pack, &run_dir)?;
    let pack_manifest = PromptPackManifest {
        name: pack.name.clone(),
        version: pack.version.clone(),
        source: pack.root.clone(),
        snapshot: snapshot.path,
        content_fingerprint: snapshot.content_fingerprint,
        file_count: snapshot.file_count,
        byte_count: snapshot.byte_count,
    };
    let manifest = RunManifest {
        run_id: run_id.clone(),
        repository: plan.repository.clone(),
        output_dir: plan.output_dir.clone(),
        run_dir: run_dir.clone(),
        config: plan.config.clone(),
        prompt_pack: pack_manifest,
        agent: plan.agent.clone(),
        model: plan.model.clone(),
        selected_pack: plan.pack.clone(),
        selected_lenses: plan.lenses.clone(),
        selected_optics: plan.optics.clone(),
        requested_domains: plan.domains.clone(),
        previous_runs: plan.previous_runs.clone(),
        dry_run: plan.dry_run,
        allow_agent_failures: plan.allow_agent_failures,
        jobs: plan.jobs,
        retries: plan.retries,
    };
    write_json_yaml(&run_dir.join("run.yaml"), &manifest)?;

    let repository = collect_repository_context(&plan.repository)?;
    let (repository_md_path, _) = write_repository_artifacts(&run_dir, &repository)?;
    let repository_context = repository_markdown(&repository);
    let domains = infer_domains(&repository, &plan.domains);
    validate_unique_domain_ids(&domains)?;
    let domain_map_context = domain_map_markdown(&domains);
    let selected_lenses = lenses_from_ids(&plan.lenses);
    let selected_optics = optics_from_ids(&plan.optics);

    let mode = if plan.dry_run {
        RunnerMode::DryRun {
            agent_name: plan.agent.clone(),
        }
    } else {
        RunnerMode::Real(Arc::new(load_effective_agent_config(
            &plan.agent,
            plan.config.as_deref(),
            &plan.repository,
        )?))
    };
    let progress_display = if plan.jobs > 1 && matches!(options.progress, ProgressDisplay::Spinner)
    {
        ProgressDisplay::Lines
    } else {
        options.progress
    };
    let mut runner = StepRunner {
        mode,
        repository: plan.repository.clone(),
        run_dir: run_dir.clone(),
        allow_failure: plan.allow_agent_failures,
        progress: RunProgress {
            display: progress_display,
            total: total_step_count(&domains, &selected_lenses, &selected_optics),
        },
        verbose_agent_output: options.verbose_agent_output,
        retries: plan.retries,
        prompt_max_chars: project_config
            .prompt_max_chars
            .unwrap_or(DEFAULT_PROMPT_MAX_CHARS),
        counter: 0,
        steps: Vec::new(),
    };

    let base_reviewer_guidance = read_pack_text(&pack.prompt("base-reviewer"))?;

    run_domain_discovery(
        &mut runner,
        &pack,
        &base_reviewer_guidance,
        &repository_context,
        &domains,
    )?;

    let job_context = Arc::new(AuditJobContext {
        pack: pack.clone(),
        base_reviewer_guidance: base_reviewer_guidance.clone(),
        repository_context: repository_context.clone(),
        domain_map_context: domain_map_context.clone(),
        previous_runs: plan.previous_runs.clone(),
        pack_policy: pack_policy.clone(),
    });
    let planned_jobs = plan_audit_jobs(&mut runner, &domains, &selected_lenses, &selected_optics)?;
    let final_report = planned_jobs.final_report.clone();

    runner.run_jobs(job_context, planned_jobs.jobs, plan.jobs)?;

    ensure_final_report(&run_dir, &manifest, &domains, &final_report)?;
    write_text_if_absent(
        &run_dir.join("suggestions/prompt-improvements.md"),
        "# Prompt Improvements\n\nNo prompt improvements were accepted during this run.\n",
        false,
    )?;
    write_text_if_absent(
        &run_dir.join("suggestions/practice-improvements.md"),
        "# Practice Improvements\n\nNo practice improvements were accepted during this run.\n",
        false,
    )?;
    copy_dir_recursive(
        &pack.root.join("prompts"),
        &run_dir.join("prompts/templates"),
    )?;

    let summary = RunSummary {
        run_id,
        run_dir,
        final_report,
        domains: domains
            .iter()
            .map(|domain| domain.domain_id.clone())
            .collect(),
        lenses: plan.lenses,
        optics: plan.optics,
        model: plan.model,
        dry_run: plan.dry_run,
        jobs: plan.jobs,
        retries: plan.retries,
        steps: runner.steps,
    };
    write_json_yaml(&summary.run_dir.join("summary.yaml"), &summary)?;
    let _ = repository_md_path;
    Ok(summary)
}

struct StepRunner {
    mode: RunnerMode,
    repository: PathBuf,
    run_dir: PathBuf,
    allow_failure: bool,
    progress: RunProgress,
    verbose_agent_output: bool,
    retries: u8,
    prompt_max_chars: usize,
    counter: usize,
    steps: Vec<AgentStepRecord>,
}

#[derive(Clone)]
enum RunnerMode {
    Real(Arc<AgentConfig>),
    DryRun { agent_name: String },
}

impl StepRunner {
    fn plan_step(
        &mut self,
        slug: &str,
        role: &str,
        report_path: PathBuf,
        findings_path: PathBuf,
    ) -> PlannedStep {
        self.counter += 1;
        let step_id = format!("{:03}-{}", self.counter, sanitize_id(slug));
        let raw_dir = self.run_dir.join("raw").join(&step_id);
        let notes_path = raw_dir.join("reviewer-notes.md");
        PlannedStep {
            ordinal: self.counter,
            step_id,
            role: role.to_owned(),
            raw_dir,
            report_path,
            findings_path,
            notes_path,
        }
    }

    fn run(
        &mut self,
        slug: &str,
        role: &str,
        prompt: String,
        report_path: PathBuf,
        findings_path: PathBuf,
    ) -> Result<AgentStepRecord> {
        let planned = self.plan_step(slug, role, report_path, findings_path);
        let record = self.executor().run_prompt(&planned, prompt)?;
        self.steps.push(record.clone());
        Ok(record)
    }

    fn run_jobs(
        &mut self,
        context: Arc<AuditJobContext>,
        jobs: Vec<StepJob>,
        max_jobs: usize,
    ) -> Result<()> {
        validate_unique_job_paths(&jobs)?;

        let records = run_job_graph(self.executor(), context, jobs, max_jobs)?;
        self.steps.extend(records);
        Ok(())
    }

    fn executor(&self) -> StepExecutor {
        StepExecutor {
            mode: self.mode.clone(),
            repository: self.repository.clone(),
            allow_failure: self.allow_failure,
            progress: self.progress,
            verbose_agent_output: self.verbose_agent_output,
            retries: self.retries,
            prompt_max_chars: self.prompt_max_chars,
        }
    }
}

#[derive(Debug, Clone)]
struct PlannedStep {
    ordinal: usize,
    step_id: String,
    role: String,
    raw_dir: PathBuf,
    report_path: PathBuf,
    findings_path: PathBuf,
    notes_path: PathBuf,
}

#[derive(Clone)]
struct StepExecutor {
    mode: RunnerMode,
    repository: PathBuf,
    allow_failure: bool,
    progress: RunProgress,
    verbose_agent_output: bool,
    retries: u8,
    prompt_max_chars: usize,
}

impl StepExecutor {
    fn run_prompt(&self, planned: &PlannedStep, prompt: String) -> Result<AgentStepRecord> {
        let prompt = limit_prompt_chars(&prompt, self.prompt_max_chars);
        let request = AgentRunRequest {
            step_id: planned.step_id.clone(),
            role: planned.role.clone(),
            cwd: self.repository.clone(),
            prompt,
            raw_dir: planned.raw_dir.clone(),
            report_path: planned.report_path.clone(),
            findings_path: planned.findings_path.clone(),
            notes_path: planned.notes_path.clone(),
            allow_failure: self.allow_failure,
        };
        let activity = self
            .progress
            .step_started(planned.ordinal, &request.step_id, &planned.role);
        let attempts = usize::from(self.retries) + 1;
        let mut last_error = None;
        let artifact_root = run_artifact_root(&planned.raw_dir);

        for attempt in 1..=attempts {
            cleanup_stale_step_outputs(&request)?;
            let repository_before =
                repository_status_snapshot(&self.repository, artifact_root.as_deref())?;
            let attempt_result = self.run_attempt(&request);
            ensure_repository_unchanged(
                &self.repository,
                artifact_root.as_deref(),
                repository_before.as_deref(),
                &request.step_id,
            )?;

            match attempt_result {
                Ok(record) if record.exit.success || attempt == attempts => {
                    let output_result = ensure_step_outputs(&record);
                    activity.finish(record.exit.success && output_result.is_ok());
                    output_result?;
                    return Ok(record);
                }
                Ok(record) => {
                    last_error = record.exit.error.clone().map(anyhow::Error::msg);
                }
                Err(error) if attempt == attempts => {
                    activity.finish(false);
                    return Err(error);
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }
        }

        activity.finish(false);
        Err(last_error.unwrap_or_else(|| anyhow!("agent step failed")))
    }

    fn run_attempt(&self, request: &AgentRunRequest) -> Result<AgentStepRecord> {
        match &self.mode {
            RunnerMode::Real(agent) => run_agent(
                agent,
                request,
                AgentRunOptions {
                    live_output: self.verbose_agent_output,
                },
            ),
            RunnerMode::DryRun { agent_name } => run_agent_dry_run(agent_name, request),
        }
    }
}

#[derive(Debug, Clone)]
struct StepJob {
    id: usize,
    dependencies: Vec<usize>,
    planned: PlannedStep,
    kind: StepJobKind,
}

#[derive(Debug, Clone)]
enum StepJobKind {
    LensReview {
        domain: Domain,
        lens: Lens,
    },
    ProjectOpticReview {
        optic: Optic,
    },
    CrossSystemReview,
    DomainSynthesis {
        domain: Domain,
        review_reports: Vec<PathBuf>,
        review_findings: Vec<PathBuf>,
    },
    SystemSynthesis {
        domain_synthesis_reports: Vec<PathBuf>,
        cross_system_report: PathBuf,
        cross_system_findings: PathBuf,
    },
    PreviousRunsComparison {
        current_findings: Vec<PathBuf>,
    },
    FinalEditor {
        system_report: PathBuf,
        domain_synthesis_reports: Vec<PathBuf>,
        cross_system_report: PathBuf,
        cross_system_findings: PathBuf,
        previous_report: PathBuf,
    },
}

#[derive(Debug, Clone)]
struct AuditJobContext {
    pack: Pack,
    base_reviewer_guidance: String,
    repository_context: String,
    domain_map_context: String,
    previous_runs: Vec<PathBuf>,
    pack_policy: PackPolicy,
}

struct PlannedAuditJobs {
    jobs: Vec<StepJob>,
    final_report: PathBuf,
}

struct JobCompletion {
    job_id: usize,
    result: Result<AgentStepRecord>,
}

fn run_job_graph(
    executor: StepExecutor,
    context: Arc<AuditJobContext>,
    jobs: Vec<StepJob>,
    max_jobs: usize,
) -> Result<Vec<AgentStepRecord>> {
    if jobs.is_empty() {
        return Ok(Vec::new());
    }

    let job_count = jobs.len();
    let mut remaining_dependencies = vec![0_usize; job_count];
    let mut dependents = vec![Vec::<usize>::new(); job_count];
    for (index, job) in jobs.iter().enumerate() {
        if job.id != index {
            bail!("internal scheduler error: job ids are not contiguous");
        }
        remaining_dependencies[job.id] = job.dependencies.len();
        for dependency in &job.dependencies {
            if *dependency >= job_count {
                bail!(
                    "internal scheduler error: job `{}` depends on unknown job `{dependency}`",
                    job.planned.step_id
                );
            }
            dependents[*dependency].push(job.id);
        }
    }

    let mut ready = VecDeque::new();
    for job in &jobs {
        if remaining_dependencies[job.id] == 0 {
            push_ready_job(&mut ready, job.id, &jobs);
        }
    }

    let (sender, receiver) = mpsc::channel::<JobCompletion>();
    let max_jobs = max_jobs.max(1);
    let mut records = vec![None; job_count];
    let mut running = 0_usize;
    let mut completed = 0_usize;
    let mut first_error = None;

    while completed < job_count {
        while running < max_jobs && first_error.is_none() {
            let Some(job_id) = ready.pop_front() else {
                break;
            };
            spawn_step_job(
                executor.clone(),
                Arc::clone(&context),
                jobs[job_id].clone(),
                sender.clone(),
            );
            running += 1;
        }

        if running == 0 {
            break;
        }

        let completion = receiver
            .recv()
            .context("receive completed audit job from worker")?;
        running -= 1;
        completed += 1;

        match completion.result {
            Ok(record) => {
                records[completion.job_id] = Some(record);
                for dependent in &dependents[completion.job_id] {
                    remaining_dependencies[*dependent] -= 1;
                    if remaining_dependencies[*dependent] == 0 {
                        push_ready_job(&mut ready, *dependent, &jobs);
                    }
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }

        if first_error.is_some() && running == 0 {
            break;
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }

    if completed != job_count {
        bail!("audit job graph stalled before all dependencies were satisfied");
    }

    let mut ordered_records = Vec::with_capacity(job_count);
    for job in &jobs {
        let Some(record) = records[job.id].take() else {
            bail!(
                "internal scheduler error: job `{}` completed without a record",
                job.planned.step_id
            );
        };
        ordered_records.push(record);
    }
    Ok(ordered_records)
}

fn push_ready_job(ready: &mut VecDeque<usize>, job_id: usize, jobs: &[StepJob]) {
    let ordinal = jobs[job_id].planned.ordinal;
    let position = ready
        .iter()
        .position(|existing| jobs[*existing].planned.ordinal > ordinal)
        .unwrap_or(ready.len());
    ready.insert(position, job_id);
}

fn spawn_step_job(
    executor: StepExecutor,
    context: Arc<AuditJobContext>,
    job: StepJob,
    sender: Sender<JobCompletion>,
) {
    thread::spawn(move || {
        let job_id = job.id;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute_step_job(&executor, &context, &job)
        }))
        .unwrap_or_else(|_| Err(anyhow!("audit job worker panicked")));
        let _ = sender.send(JobCompletion { job_id, result });
    });
}

fn execute_step_job(
    executor: &StepExecutor,
    context: &AuditJobContext,
    job: &StepJob,
) -> Result<AgentStepRecord> {
    match &job.kind {
        StepJobKind::LensReview { domain, lens } => {
            let prompt = build_lens_review_prompt(context, &job.planned, domain, *lens)?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(
                &job.planned.report_path,
                &format!("{} / {}", domain.name, lens.title()),
            )?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::ProjectOpticReview { optic } => {
            let prompt = build_project_optic_review_prompt(context, &job.planned, *optic)?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(
                &job.planned.report_path,
                &format!("{} Project Optic", optic.title()),
            )?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::CrossSystemReview => {
            let prompt = build_cross_system_review_prompt(context, &job.planned)?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(&job.planned.report_path, "Cross-System Review")?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::DomainSynthesis {
            domain,
            review_reports,
            review_findings,
        } => {
            let prompt = build_domain_synthesis_prompt(
                context,
                &job.planned,
                domain,
                review_reports,
                review_findings,
            )?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(
                &job.planned.report_path,
                &format!("{} Synthesis", domain.name),
            )?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::SystemSynthesis {
            domain_synthesis_reports,
            cross_system_report,
            cross_system_findings,
        } => {
            let prompt = build_system_synthesis_prompt(
                context,
                &job.planned,
                domain_synthesis_reports,
                cross_system_report,
                cross_system_findings,
            )?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(&job.planned.report_path, "System Review")?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::PreviousRunsComparison { current_findings } => {
            let prompt =
                build_previous_runs_comparison_prompt(context, &job.planned, current_findings)?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_review_report(&job.planned.report_path, "Previous Runs Comparison")?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
        StepJobKind::FinalEditor {
            system_report,
            domain_synthesis_reports,
            cross_system_report,
            cross_system_findings,
            previous_report,
        } => {
            let prompt = build_final_editor_prompt(
                context,
                &job.planned,
                system_report,
                domain_synthesis_reports,
                cross_system_report,
                cross_system_findings,
                previous_report,
            )?;
            let record = executor.run_prompt(&job.planned, prompt)?;
            ensure_valid_findings(&job.planned.findings_path)?;
            Ok(record)
        }
    }
}

fn validate_unique_job_paths(jobs: &[StepJob]) -> Result<()> {
    let mut seen = BTreeMap::<PathBuf, String>::new();
    for job in jobs {
        for path in [
            &job.planned.raw_dir,
            &job.planned.report_path,
            &job.planned.findings_path,
            &job.planned.notes_path,
        ] {
            if let Some(existing) = seen.insert(path.clone(), job.planned.step_id.clone()) {
                bail!(
                    "planned audit jobs `{existing}` and `{}` would write the same path {}",
                    job.planned.step_id,
                    path.display()
                );
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct RunProgress {
    display: ProgressDisplay,
    total: usize,
}

impl RunProgress {
    fn step_started(self, current: usize, step_id: &str, role: &str) -> StepActivity {
        let activity = StepActivity {
            display: self.display,
            current,
            total: self.total,
            step_id: step_id.to_owned(),
            role: role.to_owned(),
            started: Instant::now(),
            spinner: None,
        };

        match self.display {
            ProgressDisplay::Hidden => activity,
            ProgressDisplay::Lines => {
                anstream::eprintln!("start [{}/{}] {} ({})", current, self.total, role, step_id);
                activity
            }
            ProgressDisplay::Spinner => activity.with_spinner(),
        }
    }
}

struct StepActivity {
    display: ProgressDisplay,
    current: usize,
    total: usize,
    step_id: String,
    role: String,
    started: Instant,
    spinner: Option<SpinnerThread>,
}

impl StepActivity {
    fn with_spinner(mut self) -> Self {
        let (stop, receiver) = mpsc::channel();
        let current = self.current;
        let total = self.total;
        let step_id = self.step_id.clone();
        let role = self.role.clone();
        let started = self.started;
        let handle = thread::spawn(move || {
            let frames = ["-", "\\", "|", "/"];
            let mut frame = 0;

            loop {
                match receiver.recv_timeout(Duration::from_millis(250)) {
                    Ok(success) => {
                        let status = if success { "done" } else { "failed" };
                        let line = spinner_status_line(
                            status,
                            current,
                            total,
                            &role,
                            &step_id,
                            format_duration(started.elapsed()),
                        );
                        write_spinner_line(&line, true);
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        let line = spinner_status_line(
                            frames[frame % frames.len()],
                            current,
                            total,
                            &role,
                            &step_id,
                            format_duration(started.elapsed()),
                        );
                        write_spinner_line(&line, false);
                        frame += 1;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        self.spinner = Some(SpinnerThread { stop, handle });
        self
    }

    fn finish(mut self, success: bool) {
        if let Some(spinner) = self.spinner.take() {
            spinner.finish(success);
            return;
        }

        if matches!(self.display, ProgressDisplay::Lines) {
            let status = if success { "done" } else { "failed" };
            anstream::eprintln!(
                "{status} [{}/{}] {} ({}) in {}",
                self.current,
                self.total,
                self.role,
                self.step_id,
                format_duration(self.started.elapsed())
            );
        }
    }
}

fn spinner_status_line(
    status: &str,
    current: usize,
    total: usize,
    role: &str,
    step_id: &str,
    elapsed: String,
) -> String {
    let line = if matches!(status, "-" | "\\" | "|" | "/") {
        format!("{status} [{current}/{total}] {role} ({step_id}) running {elapsed}")
    } else {
        format!("{status} [{current}/{total}] {role} ({step_id}) in {elapsed}")
    };
    fit_spinner_line(&line, spinner_line_width())
}

fn write_spinner_line(line: &str, newline: bool) {
    let mut stderr = io::stderr().lock();
    let _ = write!(stderr, "\r\x1b[2K{line}");
    if newline {
        let _ = stderr.write_all(b"\n");
    }
    let _ = stderr.flush();
}

fn spinner_line_width() -> usize {
    terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .filter(|width| *width > 0)
        .unwrap_or(80)
}

fn fit_spinner_line(line: &str, terminal_width: usize) -> String {
    let max_width = terminal_width.saturating_sub(1);
    if max_width == 0 {
        return String::new();
    }

    let chars = line.chars().collect::<Vec<_>>();
    if chars.len() <= max_width {
        return line.to_owned();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let keep = max_width - 3;
    let head_len = keep * 2 / 3;
    let tail_len = keep - head_len;

    let mut fitted = chars.iter().take(head_len).copied().collect::<String>();
    fitted.push_str("...");
    fitted.extend(chars.iter().skip(chars.len() - tail_len).copied());
    fitted
}

struct SpinnerThread {
    stop: Sender<bool>,
    handle: JoinHandle<()>,
}

impl SpinnerThread {
    fn finish(self, success: bool) {
        let _ = self.stop.send(success);
        let _ = self.handle.join();
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds >= 60 {
        format!("{}m{:02}s", seconds / 60, seconds % 60)
    } else {
        format!("{}.{:01}s", seconds, duration.subsec_millis() / 100)
    }
}

fn plan_audit_jobs(
    runner: &mut StepRunner,
    domains: &[Domain],
    selected_lenses: &[Lens],
    selected_optics: &[Optic],
) -> Result<PlannedAuditJobs> {
    let mut jobs = Vec::new();
    let mut domain_review_reports = BTreeMap::<String, Vec<PathBuf>>::new();
    let mut domain_review_findings = BTreeMap::<String, Vec<PathBuf>>::new();
    let mut domain_review_job_ids = BTreeMap::<String, Vec<usize>>::new();
    let mut review_job_ids = Vec::new();
    let mut all_findings = Vec::new();

    for domain in domains {
        for lens in selected_lenses {
            let report_path = runner.run_dir.join("reports/lenses").join(format!(
                "{}.{}.md",
                domain.domain_id,
                lens.as_str()
            ));
            let findings_path = runner.run_dir.join("findings").join(format!(
                "{}.{}.yaml",
                domain.domain_id,
                lens.as_str()
            ));
            let planned = runner.plan_step(
                &format!("{}-{}", domain.domain_id, lens.as_str()),
                &format!("{} lens review", lens.as_str()),
                report_path.clone(),
                findings_path.clone(),
            );
            let job_id = jobs.len();
            jobs.push(StepJob {
                id: job_id,
                dependencies: Vec::new(),
                planned,
                kind: StepJobKind::LensReview {
                    domain: domain.clone(),
                    lens: *lens,
                },
            });
            domain_review_reports
                .entry(domain.domain_id.clone())
                .or_default()
                .push(report_path);
            domain_review_findings
                .entry(domain.domain_id.clone())
                .or_default()
                .push(findings_path.clone());
            domain_review_job_ids
                .entry(domain.domain_id.clone())
                .or_default()
                .push(job_id);
            review_job_ids.push(job_id);
            all_findings.push(findings_path);
        }
    }

    for optic in selected_optics {
        let report_path = runner
            .run_dir
            .join("reports/optics")
            .join(format!("{}.md", optic.as_str()));
        let findings_path = runner
            .run_dir
            .join("findings")
            .join(format!("{}.yaml", optic.as_str()));
        let planned = runner.plan_step(
            optic.as_str(),
            &format!("{} project optic review", optic.as_str()),
            report_path,
            findings_path.clone(),
        );
        let job_id = jobs.len();
        jobs.push(StepJob {
            id: job_id,
            dependencies: Vec::new(),
            planned,
            kind: StepJobKind::ProjectOpticReview { optic: *optic },
        });
        review_job_ids.push(job_id);
        all_findings.push(findings_path);
    }

    let cross_system_report = runner.run_dir.join("reports/cross-system.md");
    let cross_system_findings = runner.run_dir.join("findings/cross-system.yaml");
    let cross_system_job_id = jobs.len();
    jobs.push(StepJob {
        id: cross_system_job_id,
        dependencies: Vec::new(),
        planned: runner.plan_step(
            "cross-system",
            "cross-system review",
            cross_system_report.clone(),
            cross_system_findings.clone(),
        ),
        kind: StepJobKind::CrossSystemReview,
    });
    review_job_ids.push(cross_system_job_id);
    all_findings.push(cross_system_findings.clone());

    let mut domain_synthesis_reports = Vec::new();
    let mut domain_synthesis_job_ids = Vec::new();
    for domain in domains {
        let report_path = runner
            .run_dir
            .join("reports/domains")
            .join(format!("{}.md", domain.domain_id));
        let findings_path = runner
            .run_dir
            .join("findings")
            .join(format!("{}.synthesis.yaml", domain.domain_id));
        let dependencies = domain_review_job_ids
            .get(&domain.domain_id)
            .cloned()
            .unwrap_or_default();
        let review_reports = domain_review_reports
            .get(&domain.domain_id)
            .cloned()
            .unwrap_or_default();
        let review_findings = domain_review_findings
            .get(&domain.domain_id)
            .cloned()
            .unwrap_or_default();
        let planned = runner.plan_step(
            &format!("{}-synthesis", domain.domain_id),
            "domain synthesis",
            report_path.clone(),
            findings_path,
        );
        let job_id = jobs.len();
        jobs.push(StepJob {
            id: job_id,
            dependencies,
            planned,
            kind: StepJobKind::DomainSynthesis {
                domain: domain.clone(),
                review_reports,
                review_findings,
            },
        });
        domain_synthesis_reports.push(report_path);
        domain_synthesis_job_ids.push(job_id);
    }

    let system_report = runner.run_dir.join("reports/system-review.md");
    let system_findings = runner.run_dir.join("findings/system.yaml");
    let mut system_dependencies = domain_synthesis_job_ids.clone();
    system_dependencies.push(cross_system_job_id);
    let system_job_id = jobs.len();
    jobs.push(StepJob {
        id: system_job_id,
        dependencies: system_dependencies,
        planned: runner.plan_step(
            "system-synthesis",
            "system synthesis",
            system_report.clone(),
            system_findings,
        ),
        kind: StepJobKind::SystemSynthesis {
            domain_synthesis_reports: domain_synthesis_reports.clone(),
            cross_system_report: cross_system_report.clone(),
            cross_system_findings: cross_system_findings.clone(),
        },
    });

    let previous_report = runner.run_dir.join("reports/previous-runs-comparison.md");
    let previous_findings = runner
        .run_dir
        .join("findings/previous-runs-comparison.yaml");
    let previous_job_id = jobs.len();
    jobs.push(StepJob {
        id: previous_job_id,
        dependencies: review_job_ids,
        planned: runner.plan_step(
            "previous-runs-comparison",
            "previous runs comparison",
            previous_report.clone(),
            previous_findings,
        ),
        kind: StepJobKind::PreviousRunsComparison {
            current_findings: all_findings,
        },
    });

    let final_report = runner.run_dir.join("reports/final-report.md");
    let final_findings = runner.run_dir.join("findings/final.yaml");
    let mut final_dependencies = domain_synthesis_job_ids;
    final_dependencies.push(system_job_id);
    final_dependencies.push(previous_job_id);
    jobs.push(StepJob {
        id: jobs.len(),
        dependencies: final_dependencies,
        planned: runner.plan_step(
            "final-editor",
            "final editorial verification",
            final_report.clone(),
            final_findings,
        ),
        kind: StepJobKind::FinalEditor {
            system_report,
            domain_synthesis_reports: domain_synthesis_reports.clone(),
            cross_system_report,
            cross_system_findings,
            previous_report,
        },
    });

    Ok(PlannedAuditJobs { jobs, final_report })
}

fn build_lens_review_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    domain: &Domain,
    lens: Lens,
) -> Result<String> {
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "reviewer notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("domain-lens-review"))?;
    let lens_prompt = read_pack_text(&context.pack.lens_prompt(lens))?;
    let practices = read_pack_text(&context.pack.lens_practices(lens))?;
    let evidence = read_pack_text(&context.pack.lens_evidence(lens))?;
    let false_positives = read_pack_text(&context.pack.lens_false_positives(lens))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    let domain_context = domain_markdown(domain);
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("lens_prompt", &lens_prompt),
            ("repository_context", context.repository_context.as_str()),
            ("domain_context", &domain_context),
            ("domain_map", context.domain_map_context.as_str()),
            ("domain_id", &domain.domain_id),
            ("domain_name", &domain.name),
            ("lens_id", lens.as_str()),
            ("lens_practices", &practices),
            ("lens_evidence", &evidence),
            ("lens_false_positives", &false_positives),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_project_optic_review_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    optic: Optic,
) -> Result<String> {
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "reviewer notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("project-optic-review"))?;
    let optic_prompt = read_pack_text(&context.pack.optic_prompt(optic))?;
    let practices = read_pack_text(&context.pack.optic_practices(optic))?;
    let evidence = read_pack_text(&context.pack.optic_evidence(optic))?;
    let false_positives = read_pack_text(&context.pack.optic_false_positives(optic))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("optic_prompt", &optic_prompt),
            ("repository_context", context.repository_context.as_str()),
            ("domain_map", context.domain_map_context.as_str()),
            ("optic_id", optic.as_str()),
            ("optic_practices", &practices),
            ("optic_evidence", &evidence),
            ("optic_false_positives", &false_positives),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_cross_system_review_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
) -> Result<String> {
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "reviewer notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("cross-system-review"))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    let cross_system_lenses = cross_system_lenses(&context.pack_policy);
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("repository_context", context.repository_context.as_str()),
            ("domain_map", context.domain_map_context.as_str()),
            ("integration_context", &integration),
            ("cross_system_lenses", &cross_system_lenses),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_domain_synthesis_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    domain: &Domain,
    review_reports: &[PathBuf],
    review_findings: &[PathBuf],
) -> Result<String> {
    let domain_findings = read_paths(review_reports, 40_000)?;
    let domain_structured_findings = read_paths(review_findings, 40_000)?;
    let domain_context = domain_markdown(domain);
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "synthesis notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("domain-synthesis"))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("domain_id", &domain.domain_id),
            ("domain_name", &domain.name),
            ("domain_context", &domain_context),
            ("domain_findings", &domain_findings),
            ("domain_structured_findings", &domain_structured_findings),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_system_synthesis_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    domain_synthesis_reports: &[PathBuf],
    cross_system_report: &Path,
    cross_system_findings: &Path,
) -> Result<String> {
    let synthesis_inputs = read_paths(domain_synthesis_reports, 80_000)?;
    let cross_system_inputs = read_paths(
        &[
            cross_system_report.to_path_buf(),
            cross_system_findings.to_path_buf(),
        ],
        40_000,
    )?;
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "system synthesis notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("system-synthesis"))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("repository_context", context.repository_context.as_str()),
            ("synthesis_inputs", &synthesis_inputs),
            ("cross_system_inputs", &cross_system_inputs),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_previous_runs_comparison_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    current_findings: &[PathBuf],
) -> Result<String> {
    let current_findings = read_paths(current_findings, 80_000)?;
    let previous_run_context = previous_run_context(&context.previous_runs)?;
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "comparison notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("previous-runs-comparison"))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("current_findings", &current_findings),
            ("previous_run_context", &previous_run_context),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn build_final_editor_prompt(
    context: &AuditJobContext,
    planned: &PlannedStep,
    system_report: &Path,
    domain_synthesis_reports: &[PathBuf],
    cross_system_report: &Path,
    cross_system_findings: &Path,
    previous_report: &Path,
) -> Result<String> {
    let mut inputs = String::new();
    inputs.push_str(&read_optional(system_report)?.unwrap_or_default());
    inputs.push_str("\n\n");
    inputs.push_str(&read_paths(domain_synthesis_reports, 100_000)?);
    inputs.push_str("\n\n# Cross-System Inputs\n\n");
    inputs.push_str(&read_paths(
        &[
            cross_system_report.to_path_buf(),
            cross_system_findings.to_path_buf(),
        ],
        40_000,
    )?);
    inputs.push_str("\n\n");
    inputs.push_str(&read_optional(previous_report)?.unwrap_or_default());
    let output_paths = output_paths(
        &planned.report_path,
        &planned.findings_path,
        "final editor notes in raw dir",
    );
    let template = read_pack_text(&context.pack.prompt("final-editor"))?;
    let integration =
        integration_context(&context.pack, context.pack_policy.synthesis_integration())?;
    let final_editor_checklist =
        read_pack_text(&context.pack.integration("final-editor-checklist"))?;
    Ok(render_template(
        &template,
        &map_values([
            (
                "base_reviewer_guidance",
                context.base_reviewer_guidance.as_str(),
            ),
            ("final_editor_inputs", &truncate_chars(&inputs, 120_000)),
            ("integration_context", &integration),
            ("final_editor_checklist", &final_editor_checklist),
            ("output_paths", &output_paths),
        ]),
    ))
}

fn run_domain_discovery(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    repository_context: &str,
    domains: &[Domain],
) -> Result<()> {
    let report_path = runner.run_dir.join("domain-map.md");
    let findings_path = runner.run_dir.join("domain-map.yaml");
    let output_paths = output_paths(&report_path, &findings_path, "domain-map notes in raw dir");
    let template = read_pack_text(&pack.prompt("domain-discovery"))?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("repository_context", repository_context),
            ("output_paths", &output_paths),
        ]),
    );
    let _ = runner.run(
        "domain-discovery",
        "domain discovery",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;

    let domain_yaml_is_empty_findings = read_optional(&findings_path)?
        .map(|contents| contents.trim() == "[]")
        .unwrap_or(true);
    if !report_path.exists() {
        write_text(&report_path, domain_map_markdown(domains))?;
    }
    if !findings_path.exists() || domain_yaml_is_empty_findings {
        write_json_yaml(
            &findings_path,
            &DomainMap {
                domains: domains.to_vec(),
            },
        )?;
    }

    Ok(())
}

struct ResolvedPack {
    pack: Pack,
    policy: PackPolicy,
}

fn resolve_pack(plan: &AuditPlan, configured_source: Option<&Path>) -> Result<ResolvedPack> {
    let root = if let Some(configured_source) = configured_source {
        configured_source.to_path_buf()
    } else {
        let current = pack_root(&plan.prompt_home, &plan.pack_name, &plan.pack_version);
        if current.exists() {
            current
        } else {
            let legacy_named =
                legacy_pack_root(&plan.prompt_home, &plan.pack_name, &plan.pack_version);
            let legacy_default =
                legacy_pack_root(&plan.prompt_home, "ultraudit-default", &plan.pack_version);
            if legacy_named.exists() {
                legacy_named
            } else if legacy_default.exists() {
                legacy_default
            } else {
                current
            }
        }
    };

    if !root.exists() {
        bail!(
            "prompt pack `{}` version `{}` was not found at {}; run `make install` from the Ultraudit repository to install packs/{}",
            plan.pack_name,
            plan.pack_version,
            root.display(),
            plan.pack_version
        );
    }

    let pack = Pack {
        name: plan.pack_name.clone(),
        version: plan.pack_version.clone(),
        root,
    };
    let policy = read_pack_policy(&pack.root, Some(&plan.pack_version))?;
    validate_pack_files(&pack, &policy)?;

    Ok(ResolvedPack { pack, policy })
}

fn resolve_output_dir(
    args: &RunArgs,
    configured: &Option<PathBuf>,
    repository: &Path,
    cwd: &Path,
) -> PathBuf {
    if args.output_dir == Path::new(".audit-runs") {
        if let Some(path) = configured {
            return resolve_path(path, repository);
        }
    }

    resolve_path(&args.output_dir, cwd)
}

fn selected_pack_name(args: &RunArgs, configured: &Option<String>) -> String {
    if args.pack_name == "default" {
        configured.clone().unwrap_or_else(|| args.pack_name.clone())
    } else {
        args.pack_name.clone()
    }
}

fn selected_pack_version(args: &RunArgs, configured: &Option<String>) -> String {
    if args.pack_version == "0.2.0" {
        configured
            .clone()
            .unwrap_or_else(|| args.pack_version.clone())
    } else {
        args.pack_version.clone()
    }
}

fn integration_context(pack: &Pack, integration_files: &[String]) -> Result<String> {
    let mut output = String::new();
    for name in integration_files {
        output.push_str(&format!("\n\n## {name}\n\n"));
        output.push_str(&read_pack_text(&pack.integration(name))?);
    }
    Ok(output)
}

fn cross_system_lenses(pack_policy: &PackPolicy) -> String {
    pack_policy
        .cross_system_lenses()
        .iter()
        .map(|lens| lens.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn selected_agent(args: &RunArgs, configured: &Option<String>) -> String {
    if args.agent == "codex" {
        configured.clone().unwrap_or_else(|| args.agent.clone())
    } else {
        args.agent.clone()
    }
}

fn selected_lenses(args: &RunArgs, pack_policy: &PackPolicy) -> Vec<Lens> {
    if !args.lenses.is_empty() {
        args.lenses.clone()
    } else if !args.optics.is_empty() {
        Vec::new()
    } else {
        pack_policy
            .lenses_for_set(args.pack.as_str())
            .map(<[Lens]>::to_vec)
            .unwrap_or_else(|| args.pack.lenses().to_vec())
    }
}

fn selected_optics(args: &RunArgs, disabled: &[String], pack_policy: &PackPolicy) -> Vec<Optic> {
    let mut optics = if !args.optics.is_empty() {
        args.optics.clone()
    } else if !args.lenses.is_empty() {
        Vec::new()
    } else {
        pack_policy.project_optics().to_vec()
    };

    optics.retain(|optic| !disabled.iter().any(|disabled| disabled == optic.as_str()));
    optics
}

fn selected_previous_runs(args: &RunArgs, output_dir: &Path) -> Result<Vec<PathBuf>> {
    if !args.previous_runs.is_empty() {
        return Ok(args.previous_runs.clone());
    }

    if args.no_previous_runs || !output_dir.exists() {
        return Ok(Vec::new());
    }

    let mut runs = fs::read_dir(output_dir)
        .with_context(|| format!("read {}", output_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("run.yaml").exists())
        .collect::<Vec<_>>();
    runs.sort();
    let keep_from = runs.len().saturating_sub(3);
    Ok(runs.split_off(keep_from))
}

fn total_step_count(domains: &[Domain], lenses: &[Lens], optics: &[Optic]) -> usize {
    1 + (domains.len() * lenses.len()) + optics.len() + 1 + domains.len() + 1 + 1 + 1
}

fn validate_unique_domain_ids(domains: &[Domain]) -> Result<()> {
    let mut seen = BTreeMap::<&str, &str>::new();
    for domain in domains {
        if let Some(existing) = seen.insert(&domain.domain_id, &domain.name) {
            bail!(
                "domains `{existing}` and `{}` both resolve to id `{}`; choose distinct domain names",
                domain.name,
                domain.domain_id
            );
        }
    }
    Ok(())
}

fn lenses_from_ids(ids: &[String]) -> Vec<Lens> {
    Lens::all()
        .iter()
        .copied()
        .filter(|lens| ids.iter().any(|id| id == lens.as_str()))
        .collect()
}

fn optics_from_ids(ids: &[String]) -> Vec<Optic> {
    Optic::all_default()
        .iter()
        .copied()
        .filter(|optic| ids.iter().any(|id| id == optic.as_str()))
        .collect()
}

fn output_paths(report_path: &Path, findings_path: &Path, notes: &str) -> String {
    format!(
        "report: {}\nfindings: {}\nnotes: {notes}",
        report_path.display(),
        findings_path.display()
    )
}

fn limit_prompt_chars(prompt: &str, max_chars: usize) -> String {
    let char_count = prompt.chars().count();
    if max_chars == 0 || char_count <= max_chars {
        return prompt.to_owned();
    }

    let marker = "\n\n[ultraudit prompt truncated to configured character budget]\n\n";
    let marker_len = marker.chars().count();
    if max_chars <= marker_len + 2 {
        return prompt.chars().take(max_chars).collect();
    }

    let remaining = max_chars - marker_len;
    let head_len = remaining * 2 / 3;
    let tail_len = remaining - head_len;

    let mut output = prompt.chars().take(head_len).collect::<String>();
    output.push_str(marker);
    let tail = prompt
        .chars()
        .skip(char_count - tail_len)
        .collect::<String>();
    output.push_str(&tail);
    output
}

fn cleanup_stale_step_outputs(request: &AgentRunRequest) -> Result<()> {
    let paths = [
        request.report_path.clone(),
        request.findings_path.clone(),
        request.notes_path.clone(),
        request.raw_dir.join("prompt.md"),
        request.raw_dir.join("stdout.log"),
        request.raw_dir.join("stderr.log"),
        request.raw_dir.join("invocation.yaml"),
        request.raw_dir.join("command.txt"),
        request.raw_dir.join("exit.json"),
    ];
    for path in paths {
        remove_file_if_exists(&path)?;
    }
    Ok(())
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("remove stale {}", path.display())),
    }
}

fn run_artifact_root(raw_dir: &Path) -> Option<PathBuf> {
    raw_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

fn repository_status_snapshot(
    repository: &Path,
    artifact_root: Option<&Path>,
) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(repository)
        .output();
    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }

    let status = String::from_utf8_lossy(&output.stdout);
    let filtered = filter_git_status(&status, repository, artifact_root);
    Ok(Some(filtered))
}

fn ensure_repository_unchanged(
    repository: &Path,
    artifact_root: Option<&Path>,
    before: Option<&str>,
    step_id: &str,
) -> Result<()> {
    let Some(before) = before else {
        return Ok(());
    };
    let Some(after) = repository_status_snapshot(repository, artifact_root)? else {
        return Ok(());
    };
    if after == before {
        return Ok(());
    }

    bail!(
        "agent step `{step_id}` modified repository files outside run artifacts; repository reviewer steps must be read-only"
    )
}

fn filter_git_status(status: &str, repository: &Path, artifact_root: Option<&Path>) -> String {
    let allowed_prefix = artifact_root.and_then(|path| {
        path.strip_prefix(repository)
            .ok()
            .map(relative_display_string)
    });

    status
        .lines()
        .filter(|line| {
            let path = line.get(3..).unwrap_or_default();
            !allowed_prefix
                .as_deref()
                .is_some_and(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn relative_display_string(path: &Path) -> String {
    path.display().to_string()
}

fn ensure_step_outputs(record: &AgentStepRecord) -> Result<()> {
    if !record.invocation.notes_path.exists() {
        write_text(
            &record.invocation.notes_path,
            format!(
                "# Reviewer Notes\n\nNo reviewer notes artifact was written by step `{}`.\n",
                record.invocation.step_id
            ),
        )?;
    }

    Ok(())
}

fn ensure_review_report(path: &Path, title: &str) -> Result<()> {
    if path.exists() && path.metadata().map(|metadata| metadata.len()).unwrap_or(0) > 0 {
        return Ok(());
    }

    write_text(
        path,
        format!(
            "# {title}\n\nNo markdown report was emitted by the agent. Raw stdout, stderr, prompt, invocation, and exit metadata are preserved under the matching `raw/` step directory.\n"
        ),
    )
}

fn ensure_valid_findings(path: &Path) -> Result<()> {
    if path.exists() {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        validate_findings_contract(path, &contents)?;
        return Ok(());
    }

    write_text(path, "[]\n")
}

fn validate_findings_contract(path: &Path, contents: &str) -> Result<()> {
    let trimmed = contents.trim();
    if trimmed == "[]" || trimmed == r"[]\" {
        return Ok(());
    }

    if !(trimmed.starts_with("- ") || trimmed.starts_with('[')) {
        bail!(
            "findings file {} must be a YAML/JSON sequence or []",
            path.display()
        );
    }

    for required in ["id", "title", "severity", "confidence", "recommendation"] {
        if !trimmed.contains(required) {
            bail!(
                "findings file {} is missing required finding field `{required}`",
                path.display()
            );
        }
    }

    Ok(())
}

fn ensure_final_report(
    run_dir: &Path,
    manifest: &RunManifest,
    domains: &[Domain],
    final_report: &Path,
) -> Result<()> {
    if final_report.exists()
        && final_report
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or(0)
            > 0
    {
        return Ok(());
    }

    let mut output = String::new();
    output.push_str("# Ultraudit Final Report\n\n");
    output.push_str(&format!("- run id: `{}`\n", manifest.run_id));
    output.push_str(&format!(
        "- repository: `{}`\n",
        manifest.repository.display()
    ));
    output.push_str(&format!(
        "- prompt pack: `{}` `{}`\n",
        manifest.prompt_pack.name, manifest.prompt_pack.version
    ));
    output.push_str(&format!("- agent: `{}`\n", manifest.agent));
    output.push_str(&format!(
        "- model: `{}`\n\n",
        manifest.model.as_deref().unwrap_or("auto")
    ));
    output.push_str("## Reviewed Domains\n\n");
    for domain in domains {
        output.push_str(&format!("- `{}` - {}\n", domain.domain_id, domain.name));
    }
    output.push_str("\n## System Review\n\n");
    output.push_str(
        &read_optional(&run_dir.join("reports/system-review.md"))?
            .unwrap_or_else(|| "No system synthesis report was emitted by the agent.\n".to_owned()),
    );
    output.push_str("\n\n## Previous Runs\n\n");
    output.push_str(
        &read_optional(&run_dir.join("reports/previous-runs-comparison.md"))?
            .unwrap_or_else(|| "No previous-run comparison was emitted by the agent.\n".to_owned()),
    );
    write_text(final_report, output)
}

fn read_paths(paths: &[PathBuf], max_chars: usize) -> Result<String> {
    let mut output = String::new();
    for path in paths {
        output.push_str(&format!("\n\n## `{}`\n\n", path.display()));
        output.push_str(&read_optional(path)?.unwrap_or_else(|| "[missing]\n".to_owned()));
    }
    Ok(truncate_chars(&output, max_chars))
}

fn previous_run_context(paths: &[PathBuf]) -> Result<String> {
    if paths.is_empty() {
        return Ok("No previous Ultraudit runs were selected or discovered.\n".to_owned());
    }

    let mut output = String::new();
    for path in paths {
        output.push_str(&format!("\n\n# Previous Run `{}`\n\n", path.display()));
        output.push_str(
            &read_optional(&path.join("reports/final-report.md"))?
                .unwrap_or_else(|| "No final report found.\n".to_owned()),
        );
        output.push_str("\n\n## Findings Files\n\n");
        let findings_dir = path.join("findings");
        if findings_dir.exists() {
            for entry in fs::read_dir(&findings_dir)
                .with_context(|| format!("read {}", findings_dir.display()))?
            {
                let entry =
                    entry.with_context(|| format!("read entry in {}", findings_dir.display()))?;
                let file_path = entry.path();
                if file_path.is_file() {
                    output.push_str(&format!("\n### `{}`\n\n", file_path.display()));
                    output.push_str(&read_optional(&file_path)?.unwrap_or_default());
                }
            }
        }
    }

    Ok(truncate_chars(&output, 100_000))
}

fn prune_old_runs(output_dir: &Path, retention_days: u64) -> Result<()> {
    if retention_days == 0 || !output_dir.exists() {
        return Ok(());
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(Duration::from_secs(retention_days.saturating_mul(86_400)));
    let Some(cutoff) = cutoff else {
        return Ok(());
    };

    for entry in
        fs::read_dir(output_dir).with_context(|| format!("read {}", output_dir.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", output_dir.display()))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("run.yaml").exists() {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .with_context(|| format!("metadata {}", path.display()))?;
        if modified < cutoff {
            fs::remove_dir_all(&path)
                .with_context(|| format!("remove expired audit run {}", path.display()))?;
        }
    }

    Ok(())
}

fn map_values<'a>(
    items: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> BTreeMap<&'a str, String> {
    items
        .into_iter()
        .map(|(key, value)| (key, value.to_owned()))
        .collect()
}

fn project_config_template(pack_version: &str, pack_root: &Path) -> String {
    format!(
        r#"[prompt_pack]
version = "{pack_version}"
source = "{}"

[run]
output_dir = ".audit-runs"
agent = "codex"
disabled_optics = []

[artifacts]
# 0 or absent disables automatic retention cleanup.
retention_days = 0

[prompt]
max_chars = 180000

[models]
codex = "gpt-5.5"
"#,
        pack_root.display()
    )
}

fn codex_agent_template() -> &'static str {
    r#"kind = "codex-cli"
binary = "codex"
mode = "exec"
# Agent-local `model = "..."` overrides `.audit/config.toml` [models].
ignore_user_config = true
prompt_transport = "stdin"
approval_policy = "never"
sandbox = "workspace-write"
timeout_seconds = 7200
"#
}

fn custom_shell_agent_template() -> &'static str {
    r#"kind = "shell-template"
shell = "sh"
prompt_transport = "stdin"
timeout_seconds = 7200
command = "printf 'custom-shell agent is not configured\n' >&2; exit 2"
"#
}

#[cfg(test)]
mod progress_tests {
    use super::*;

    #[test]
    fn fit_spinner_line_leaves_room_for_carriage_return_update() {
        let line = "- [14/71] dependency-supply-chain lens review (014-docs-dependency-supply-chain) running 12.3s";

        let fitted = fit_spinner_line(line, 60);

        assert!(
            fitted.chars().count() <= 59,
            "spinner line should not reach the terminal wrap column: {fitted}"
        );
    }

    #[test]
    fn fit_spinner_line_preserves_short_status_lines() {
        let line = "- [15/71] ux-product lens review (015-docs-ux-product) running 4.0s";

        assert_eq!(fit_spinner_line(line, 120), line);
    }

    #[test]
    fn fit_spinner_line_handles_tiny_terminal_widths() {
        assert_eq!(fit_spinner_line("abcdef", 1), "");
        assert_eq!(fit_spinner_line("abcdef", 4), "...");
    }
}
