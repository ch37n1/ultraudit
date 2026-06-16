pub mod cli;

mod config;
mod intake;
mod model;
mod orchestrator;
mod pack;
mod runner;
mod toml;
mod util;

use std::{
    fmt::Display,
    io::{self, IsTerminal},
    path::PathBuf,
};

use anstyle::{AnsiColor, Style};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, ColorMode, Command, InitArgs, OutputArgs, OutputFormat, RunArgs};
use model::{AuditPlan, InitPlan, RunSummary};
use tracing_subscriber::EnvFilter;

pub fn run_from_env() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(&cli.output);
    run(cli)
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Run(args) => run_audit(args, &cli.output),
        Command::Init(args) => init_project(args, &cli.output),
        Command::Completions { shell } => {
            let mut command = Cli::command();
            let command_name = command.get_name().to_owned();
            generate(shell, &mut command, command_name, &mut io::stdout());
            Ok(())
        }
    }
}

fn init_tracing(output: &OutputArgs) {
    let default_level = match output.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(output.color.allows_color())
        .without_time()
        .try_init();
}

fn run_audit(args: RunArgs, output: &OutputArgs) -> Result<()> {
    let plan = orchestrator::build_audit_plan(&args)?;

    if args.plan {
        if matches!(output.format, OutputFormat::Json) {
            anstream::println!("{}", serde_json::to_string_pretty(&plan)?);
        } else {
            print_audit_plan(&plan, output);
            print_status("plan", "Resolved only; no files written.", output);
        }
        return Ok(());
    }

    if matches!(output.format, OutputFormat::Text) {
        print_audit_plan(&plan, output);
        if args.dry_run {
            print_status(
                "dry-run",
                "Agent calls will be faked; prompts and run artifacts will still be written.",
                output,
            );
        }
    }

    let summary = orchestrator::execute_audit(&args, audit_execution_options(output))?;
    if matches!(output.format, OutputFormat::Json) {
        anstream::println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_run_summary(&summary, output);
    }

    Ok(())
}

fn audit_execution_options(output: &OutputArgs) -> orchestrator::AuditExecutionOptions {
    let progress = if matches!(output.format, OutputFormat::Json) {
        orchestrator::ProgressDisplay::Hidden
    } else if output.verbose > 0 || !io::stderr().is_terminal() {
        orchestrator::ProgressDisplay::Lines
    } else {
        orchestrator::ProgressDisplay::Spinner
    };

    orchestrator::AuditExecutionOptions {
        progress,
        verbose_agent_output: output.verbose > 0,
    }
}

fn init_project(args: InitArgs, output: &OutputArgs) -> Result<()> {
    let plan = orchestrator::init_project(&args)?;

    if matches!(output.format, OutputFormat::Json) {
        anstream::println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print_init_plan(&plan, output);
    }

    if args.dry_run && matches!(output.format, OutputFormat::Text) {
        print_status(
            "dry-run",
            "Init locations resolved; no files written.",
            output,
        );
    }

    Ok(())
}

fn print_audit_plan(plan: &AuditPlan, output: &OutputArgs) {
    print_heading("audit plan", output);
    print_kv("repository", plan.repository.display(), output);
    print_kv("output dir", plan.output_dir.display(), output);
    print_kv("config", display_option_path(plan.config.as_ref()), output);
    print_kv("prompt home", plan.prompt_home.display(), output);
    print_kv(
        "mode",
        if plan.dry_run { "dry-run" } else { "real" },
        output,
    );
    print_kv("agent", &plan.agent, output);
    print_kv("jobs", plan.jobs, output);
    print_kv("retries", plan.retries, output);
    print_kv("pack", &plan.pack, output);
    print_kv("pack name", &plan.pack_name, output);
    print_kv("pack version", &plan.pack_version, output);
    print_kv("pack source", plan.pack_source.display(), output);
    print_kv(
        "model",
        display_option_str(plan.model.as_deref(), "auto"),
        output,
    );
    print_kv("lenses", display_list_or(&plan.lenses, "none"), output);
    print_kv("optics", display_list_or(&plan.optics, "none"), output);
    print_kv(
        "domains",
        display_list_or(&plan.domains, "all discovered domains"),
        output,
    );
    print_kv(
        "previous runs",
        display_path_list_or(&plan.previous_runs, "none"),
        output,
    );
}

fn print_init_plan(plan: &InitPlan, output: &OutputArgs) {
    print_heading("init plan", output);
    print_kv(
        "project config dir",
        plan.project_config_dir.display(),
        output,
    );
    print_kv("prompt home", plan.prompt_home.display(), output);
    print_kv("pack name", &plan.pack_name, output);
    print_kv("pack version", &plan.pack_version, output);
    print_kv("force", plan.force, output);
}

fn print_run_summary(summary: &RunSummary, output: &OutputArgs) {
    print_heading("audit complete", output);
    print_kv("run id", &summary.run_id, output);
    print_kv("run dir", summary.run_dir.display(), output);
    print_kv("final report", summary.final_report.display(), output);
    print_kv(
        "mode",
        if summary.dry_run { "dry-run" } else { "real" },
        output,
    );
    print_kv("domains", display_list_or(&summary.domains, "none"), output);
    print_kv("lenses", display_list_or(&summary.lenses, "none"), output);
    print_kv("optics", display_list_or(&summary.optics, "none"), output);
    print_kv("jobs", summary.jobs, output);
    print_kv("retries", summary.retries, output);
    print_kv("agent steps", summary.steps.len(), output);
}

fn print_heading(label: &str, output: &OutputArgs) {
    let style = Style::new().bold().fg_color(Some(AnsiColor::Cyan.into()));
    anstream::println!("{}", paint(label, style, output));
}

fn print_status(label: &str, message: &str, output: &OutputArgs) {
    let style = Style::new().bold().fg_color(Some(AnsiColor::Green.into()));
    anstream::println!("{} {}", paint(label, style, output), message);
}

fn print_kv(label: &str, value: impl Display, output: &OutputArgs) {
    let style = Style::new().bold();
    anstream::println!("  {}: {}", paint(label, style, output), value);
}

fn paint(value: &str, style: Style, output: &OutputArgs) -> String {
    if output.color.should_color_stdout() {
        format!("{style}{value}{}", anstyle::Reset)
    } else {
        value.to_owned()
    }
}

fn display_option_path(path: Option<&PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "auto".to_owned())
}

fn display_option_str(value: Option<&str>, empty: &str) -> String {
    value.unwrap_or(empty).to_owned()
}

fn display_list_or<T: Display>(items: &[T], empty: &str) -> String {
    if items.is_empty() {
        empty.to_owned()
    } else {
        items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn display_path_list_or(items: &[PathBuf], empty: &str) -> String {
    if items.is_empty() {
        empty.to_owned()
    } else {
        items
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ColorMode {
    fn allows_color(self) -> bool {
        self.should_color_stderr()
    }

    fn should_color_stdout(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Auto => std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal(),
            ColorMode::Never => false,
        }
    }

    fn should_color_stderr(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Auto => std::env::var_os("NO_COLOR").is_none() && io::stderr().is_terminal(),
            ColorMode::Never => false,
        }
    }
}
