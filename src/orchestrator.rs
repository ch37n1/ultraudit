use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    cli::{Lens, Optic, RunArgs},
    config::{find_project_config, load_agent_config, load_project_config, AgentConfig},
    intake::{
        collect_repository_context, domain_map_markdown, domain_markdown, infer_domains,
        repository_markdown, write_repository_artifacts,
    },
    model::{
        AgentStepRecord, AuditPlan, Domain, DomainMap, InitPlan, PromptPackManifest, RunManifest,
        RunSummary,
    },
    pack::{legacy_pack_root, pack_root, read_pack_text, render_template, snapshot_pack, Pack},
    runner::{run_agent, run_agent_dry_run, AgentRunRequest},
    util::{
        copy_dir_recursive, now_run_id, read_optional, resolve_path, sanitize_id, truncate_chars,
        write_json_yaml, write_text, write_text_if_absent,
    },
};

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
    let lenses = selected_lenses(args);
    let optics = selected_optics(args, &project_config.disabled_optics);
    let previous_runs = selected_previous_runs(args, &output_dir)?;

    Ok(AuditPlan {
        repository,
        output_dir,
        config,
        prompt_home,
        pack_name,
        pack_version,
        pack_source,
        pack: args.pack.as_str().to_owned(),
        agent: selected_agent(args, &project_config.default_agent),
        lenses: lenses.iter().map(|lens| lens.as_str().to_owned()).collect(),
        optics: optics
            .iter()
            .map(|optic| optic.as_str().to_owned())
            .collect(),
        domains: args.domains.clone(),
        previous_runs,
        dry_run: args.dry_run,
        allow_agent_failures: args.allow_agent_failures,
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

pub fn execute_audit(args: &RunArgs) -> Result<RunSummary> {
    let plan = build_audit_plan(args)?;
    if plan.lenses.is_empty() && plan.optics.is_empty() {
        bail!("no lenses or optics selected");
    }

    let project_config = load_project_config(plan.config.as_deref())?;
    let pack = resolve_pack(&plan, project_config.prompt_pack_source.as_deref())?;

    fs::create_dir_all(&plan.output_dir)
        .with_context(|| format!("create {}", plan.output_dir.display()))?;
    let run_id = now_run_id();
    let run_dir = plan.output_dir.join(&run_id);
    fs::create_dir_all(&run_dir).with_context(|| format!("create {}", run_dir.display()))?;

    let snapshot = snapshot_pack(&pack, &run_dir)?;
    let pack_manifest = PromptPackManifest {
        name: pack.name.clone(),
        version: pack.version.clone(),
        source: pack.root.clone(),
        snapshot,
    };
    let manifest = RunManifest {
        run_id: run_id.clone(),
        repository: plan.repository.clone(),
        output_dir: plan.output_dir.clone(),
        run_dir: run_dir.clone(),
        config: plan.config.clone(),
        prompt_pack: pack_manifest,
        agent: plan.agent.clone(),
        selected_pack: plan.pack.clone(),
        selected_lenses: plan.lenses.clone(),
        selected_optics: plan.optics.clone(),
        requested_domains: plan.domains.clone(),
        previous_runs: plan.previous_runs.clone(),
        dry_run: plan.dry_run,
        allow_agent_failures: plan.allow_agent_failures,
    };
    write_json_yaml(&run_dir.join("run.yaml"), &manifest)?;

    let repository = collect_repository_context(&plan.repository)?;
    let (repository_md_path, _) = write_repository_artifacts(&run_dir, &repository)?;
    let repository_context = repository_markdown(&repository);
    let domains = infer_domains(&repository, &plan.domains);
    let domain_map_context = domain_map_markdown(&domains);

    let mode = if plan.dry_run {
        RunnerMode::DryRun {
            agent_name: plan.agent.clone(),
        }
    } else {
        RunnerMode::Real(load_agent_config(
            &plan.agent,
            plan.config.as_deref(),
            &plan.repository,
        )?)
    };
    let mut runner = StepRunner {
        mode,
        repository: plan.repository.clone(),
        run_dir: run_dir.clone(),
        allow_failure: plan.allow_agent_failures,
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

    let selected_lenses = lenses_from_ids(&plan.lenses);
    let selected_optics = optics_from_ids(&plan.optics);
    let mut domain_review_reports = BTreeMap::<String, Vec<PathBuf>>::new();
    let mut all_findings = Vec::new();

    for domain in &domains {
        for lens in &selected_lenses {
            let (report, findings) = run_lens_review(
                &mut runner,
                &pack,
                &base_reviewer_guidance,
                &repository_context,
                &domain_map_context,
                domain,
                *lens,
            )?;
            domain_review_reports
                .entry(domain.domain_id.clone())
                .or_default()
                .push(report);
            all_findings.push(findings);
        }
    }

    for optic in &selected_optics {
        let (_report, findings) = run_project_optic_review(
            &mut runner,
            &pack,
            &base_reviewer_guidance,
            &repository_context,
            &domain_map_context,
            *optic,
        )?;
        all_findings.push(findings);
    }

    let cross_system_findings = run_cross_system_review(
        &mut runner,
        &pack,
        &base_reviewer_guidance,
        &repository_context,
        &domain_map_context,
    )?;
    all_findings.push(cross_system_findings);

    let mut domain_synthesis_reports = Vec::new();
    for domain in &domains {
        let report = run_domain_synthesis(
            &mut runner,
            &pack,
            &base_reviewer_guidance,
            domain,
            domain_review_reports
                .get(&domain.domain_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        )?;
        domain_synthesis_reports.push(report);
    }

    let system_report = run_system_synthesis(
        &mut runner,
        &pack,
        &base_reviewer_guidance,
        &repository_context,
        &domain_synthesis_reports,
    )?;
    let previous_report = run_previous_runs_comparison(
        &mut runner,
        &pack,
        &base_reviewer_guidance,
        &all_findings,
        &plan.previous_runs,
    )?;
    let final_report = run_final_editor(
        &mut runner,
        &pack,
        &base_reviewer_guidance,
        &system_report,
        &domain_synthesis_reports,
        &previous_report,
    )?;

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
        dry_run: plan.dry_run,
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
    counter: usize,
    steps: Vec<AgentStepRecord>,
}

enum RunnerMode {
    Real(AgentConfig),
    DryRun { agent_name: String },
}

impl StepRunner {
    fn run(
        &mut self,
        slug: &str,
        role: &str,
        prompt: String,
        report_path: PathBuf,
        findings_path: PathBuf,
    ) -> Result<AgentStepRecord> {
        self.counter += 1;
        let step_id = format!("{:03}-{}", self.counter, sanitize_id(slug));
        let raw_dir = self.run_dir.join("raw").join(&step_id);
        let notes_path = raw_dir.join("reviewer-notes.md");
        let request = AgentRunRequest {
            step_id,
            role: role.to_owned(),
            cwd: self.repository.clone(),
            prompt,
            raw_dir,
            report_path,
            findings_path,
            notes_path,
            allow_failure: self.allow_failure,
        };
        let record = match &self.mode {
            RunnerMode::Real(agent) => run_agent(agent, &request)?,
            RunnerMode::DryRun { agent_name } => run_agent_dry_run(agent_name, &request)?,
        };
        ensure_step_outputs(&record)?;
        self.steps.push(record.clone());
        Ok(record)
    }
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

fn run_lens_review(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    repository_context: &str,
    domain_map: &str,
    domain: &Domain,
    lens: Lens,
) -> Result<(PathBuf, PathBuf)> {
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
    let output_paths = output_paths(&report_path, &findings_path, "reviewer notes in raw dir");
    let template = read_pack_text(&pack.prompt("domain-lens-review"))?;
    let lens_prompt = read_pack_text(&pack.lens_prompt(lens))?;
    let practices = read_pack_text(&pack.lens_practices(lens))?;
    let evidence = read_pack_text(&pack.lens_evidence(lens))?;
    let false_positives = read_pack_text(&pack.lens_false_positives(lens))?;
    let integration = integration_context(pack)?;
    let domain_context = domain_markdown(domain);
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("lens_prompt", &lens_prompt),
            ("repository_context", repository_context),
            ("domain_context", &domain_context),
            ("domain_map", domain_map),
            ("domain_id", &domain.domain_id),
            ("domain_name", &domain.name),
            ("lens_id", lens.as_str()),
            ("lens_practices", &practices),
            ("lens_evidence", &evidence),
            ("lens_false_positives", &false_positives),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        &format!("{}-{}", domain.domain_id, lens.as_str()),
        &format!("{} lens review", lens.as_str()),
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, &format!("{} / {}", domain.name, lens.title()))?;
    ensure_empty_findings(&findings_path)?;
    Ok((report_path, findings_path))
}

fn run_project_optic_review(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    repository_context: &str,
    domain_map: &str,
    optic: Optic,
) -> Result<(PathBuf, PathBuf)> {
    let report_path = runner
        .run_dir
        .join("reports/optics")
        .join(format!("{}.md", optic.as_str()));
    let findings_path = runner
        .run_dir
        .join("findings")
        .join(format!("{}.yaml", optic.as_str()));
    let output_paths = output_paths(&report_path, &findings_path, "reviewer notes in raw dir");
    let template = read_pack_text(&pack.prompt("project-optic-review"))?;
    let optic_prompt = read_pack_text(&pack.optic_prompt(optic))?;
    let practices = read_pack_text(&pack.optic_practices(optic))?;
    let evidence = read_pack_text(&pack.optic_evidence(optic))?;
    let false_positives = read_pack_text(&pack.optic_false_positives(optic))?;
    let integration = integration_context(pack)?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("optic_prompt", &optic_prompt),
            ("repository_context", repository_context),
            ("domain_map", domain_map),
            ("optic_id", optic.as_str()),
            ("optic_practices", &practices),
            ("optic_evidence", &evidence),
            ("optic_false_positives", &false_positives),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        optic.as_str(),
        &format!("{} project optic review", optic.as_str()),
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, &format!("{} Project Optic", optic.title()))?;
    ensure_empty_findings(&findings_path)?;
    Ok((report_path, findings_path))
}

fn run_cross_system_review(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    repository_context: &str,
    domain_map: &str,
) -> Result<PathBuf> {
    let report_path = runner.run_dir.join("reports/cross-system.md");
    let findings_path = runner.run_dir.join("findings/cross-system.yaml");
    let output_paths = output_paths(&report_path, &findings_path, "reviewer notes in raw dir");
    let template = read_pack_text(&pack.prompt("cross-system-review"))?;
    let integration = integration_context(pack)?;
    let cross_system_lenses = cross_system_lenses();
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("repository_context", repository_context),
            ("domain_map", domain_map),
            ("integration_context", &integration),
            ("cross_system_lenses", &cross_system_lenses),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        "cross-system",
        "cross-system review",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, "Cross-System Review")?;
    ensure_empty_findings(&findings_path)?;
    Ok(findings_path)
}

fn run_domain_synthesis(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    domain: &Domain,
    review_reports: &[PathBuf],
) -> Result<PathBuf> {
    let report_path = runner
        .run_dir
        .join("reports/domains")
        .join(format!("{}.md", domain.domain_id));
    let findings_path = runner
        .run_dir
        .join("findings")
        .join(format!("{}.synthesis.yaml", domain.domain_id));
    let domain_findings = read_paths(review_reports, 40_000)?;
    let domain_context = domain_markdown(domain);
    let output_paths = output_paths(&report_path, &findings_path, "synthesis notes in raw dir");
    let template = read_pack_text(&pack.prompt("domain-synthesis"))?;
    let integration = integration_context(pack)?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("domain_id", &domain.domain_id),
            ("domain_name", &domain.name),
            ("domain_context", &domain_context),
            ("domain_findings", &domain_findings),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        &format!("{}-synthesis", domain.domain_id),
        "domain synthesis",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, &format!("{} Synthesis", domain.name))?;
    ensure_empty_findings(&findings_path)?;
    Ok(report_path)
}

fn run_system_synthesis(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    repository_context: &str,
    domain_synthesis_reports: &[PathBuf],
) -> Result<PathBuf> {
    let report_path = runner.run_dir.join("reports/system-review.md");
    let findings_path = runner.run_dir.join("findings/system.yaml");
    let synthesis_inputs = read_paths(domain_synthesis_reports, 80_000)?;
    let output_paths = output_paths(
        &report_path,
        &findings_path,
        "system synthesis notes in raw dir",
    );
    let template = read_pack_text(&pack.prompt("system-synthesis"))?;
    let integration = integration_context(pack)?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("repository_context", repository_context),
            ("synthesis_inputs", &synthesis_inputs),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        "system-synthesis",
        "system synthesis",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, "System Review")?;
    ensure_empty_findings(&findings_path)?;
    Ok(report_path)
}

fn run_previous_runs_comparison(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    current_findings: &[PathBuf],
    previous_runs: &[PathBuf],
) -> Result<PathBuf> {
    let report_path = runner.run_dir.join("reports/previous-runs-comparison.md");
    let findings_path = runner
        .run_dir
        .join("findings/previous-runs-comparison.yaml");
    let current_findings = read_paths(current_findings, 80_000)?;
    let previous_run_context = previous_run_context(previous_runs)?;
    let output_paths = output_paths(&report_path, &findings_path, "comparison notes in raw dir");
    let template = read_pack_text(&pack.prompt("previous-runs-comparison"))?;
    let integration = integration_context(pack)?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("current_findings", &current_findings),
            ("previous_run_context", &previous_run_context),
            ("integration_context", &integration),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        "previous-runs-comparison",
        "previous runs comparison",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_review_report(&report_path, "Previous Runs Comparison")?;
    ensure_empty_findings(&findings_path)?;
    Ok(report_path)
}

fn run_final_editor(
    runner: &mut StepRunner,
    pack: &Pack,
    base_reviewer_guidance: &str,
    system_report: &Path,
    domain_synthesis_reports: &[PathBuf],
    previous_report: &Path,
) -> Result<PathBuf> {
    let report_path = runner.run_dir.join("reports/final-report.md");
    let findings_path = runner.run_dir.join("findings/final.yaml");
    let mut inputs = String::new();
    inputs.push_str(&read_optional(system_report)?.unwrap_or_default());
    inputs.push_str("\n\n");
    inputs.push_str(&read_paths(domain_synthesis_reports, 100_000)?);
    inputs.push_str("\n\n");
    inputs.push_str(&read_optional(previous_report)?.unwrap_or_default());
    let output_paths = output_paths(
        &report_path,
        &findings_path,
        "final editor notes in raw dir",
    );
    let template = read_pack_text(&pack.prompt("final-editor"))?;
    let integration = integration_context(pack)?;
    let final_editor_checklist = read_pack_text(&pack.integration("final-editor-checklist"))?;
    let prompt = render_template(
        &template,
        &map_values([
            ("base_reviewer_guidance", base_reviewer_guidance),
            ("final_editor_inputs", &truncate_chars(&inputs, 120_000)),
            ("integration_context", &integration),
            ("final_editor_checklist", &final_editor_checklist),
            ("output_paths", &output_paths),
        ]),
    );

    let _ = runner.run(
        "final-editor",
        "final editorial verification",
        prompt,
        report_path.clone(),
        findings_path.clone(),
    )?;
    ensure_empty_findings(&findings_path)?;
    Ok(report_path)
}

fn resolve_pack(plan: &AuditPlan, configured_source: Option<&Path>) -> Result<Pack> {
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

    Ok(Pack {
        name: plan.pack_name.clone(),
        version: plan.pack_version.clone(),
        root,
    })
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

fn integration_context(pack: &Pack) -> Result<String> {
    let mut output = String::new();
    for name in [
        "evidence-model",
        "severity-model",
        "confidence-model",
        "deduplication-rules",
    ] {
        output.push_str(&format!("\n\n## {name}\n\n"));
        output.push_str(&read_pack_text(&pack.integration(name))?);
    }
    Ok(output)
}

fn cross_system_lenses() -> String {
    Lens::cross_system_default()
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

fn selected_lenses(args: &RunArgs) -> Vec<Lens> {
    if !args.lenses.is_empty() {
        args.lenses.clone()
    } else if !args.optics.is_empty() {
        Vec::new()
    } else {
        args.pack.lenses().to_vec()
    }
}

fn selected_optics(args: &RunArgs, disabled: &[String]) -> Vec<Optic> {
    let mut optics = if !args.optics.is_empty() {
        args.optics.clone()
    } else if !args.lenses.is_empty() {
        Vec::new()
    } else {
        Optic::all_default().to_vec()
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

fn ensure_empty_findings(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    write_text(path, "[]\n")
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
    output.push_str(&format!("- agent: `{}`\n\n", manifest.agent));
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
"#,
        pack_root.display()
    )
}

fn codex_agent_template() -> &'static str {
    r#"kind = "codex-cli"
binary = "codex"
mode = "exec"
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
