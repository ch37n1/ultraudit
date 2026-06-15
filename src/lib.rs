pub mod cli;

use std::{
    fmt::Display,
    io::{self, IsTerminal},
    path::PathBuf,
};

use anstyle::{AnsiColor, Style};
use anyhow::{bail, Result};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, ColorMode, Command, InitArgs, OutputArgs, OutputFormat, RunArgs};
use serde::Serialize;
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
    let plan = AuditPlan::from(&args);

    if matches!(output.format, OutputFormat::Json) {
        anstream::println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print_heading("audit plan", output);
        print_kv("repository", plan.repository.display(), output);
        print_kv("output dir", plan.output_dir.display(), output);
        print_kv("config", display_option_path(plan.config.as_ref()), output);
        print_kv("agent", &plan.agent, output);
        print_kv("pack", plan.pack, output);
        print_kv(
            "lenses",
            display_list_or(&plan.lenses, "selected by pack"),
            output,
        );
        print_kv(
            "optics",
            display_list_or(&plan.optics, "selected by pack defaults"),
            output,
        );
        print_kv(
            "domains",
            display_list_or(&plan.domains, "all discovered domains"),
            output,
        );
    }

    if args.dry_run {
        if matches!(output.format, OutputFormat::Text) {
            print_status(
                "dry-run",
                "CLI arguments parsed; no agent processes started.",
                output,
            );
        }
        return Ok(());
    }

    bail!("audit execution is not implemented yet; rerun with --dry-run to validate CLI input")
}

fn init_project(args: InitArgs, output: &OutputArgs) -> Result<()> {
    let plan = InitPlan::from(&args);

    if matches!(output.format, OutputFormat::Json) {
        anstream::println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print_heading("init plan", output);
        print_kv(
            "project config dir",
            plan.project_config_dir.display(),
            output,
        );
        print_kv("prompt home", plan.prompt_home.display(), output);
        print_kv("force", plan.force, output);
    }

    if args.dry_run {
        if matches!(output.format, OutputFormat::Text) {
            print_status(
                "dry-run",
                "Init locations resolved; no files written.",
                output,
            );
        }
        return Ok(());
    }

    bail!("project init is not implemented yet; rerun with --dry-run to validate paths")
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

#[derive(Debug, Serialize)]
struct AuditPlan {
    repository: PathBuf,
    output_dir: PathBuf,
    config: Option<PathBuf>,
    agent: String,
    pack: cli::LensPack,
    lenses: Vec<cli::Lens>,
    optics: Vec<cli::Optic>,
    domains: Vec<String>,
    dry_run: bool,
}

impl From<&RunArgs> for AuditPlan {
    fn from(args: &RunArgs) -> Self {
        Self {
            repository: args.repository.clone(),
            output_dir: args.output_dir.clone(),
            config: args.config.clone(),
            agent: args.agent.clone(),
            pack: args.pack,
            lenses: args.lenses.clone(),
            optics: args.optics.clone(),
            domains: args.domains.clone(),
            dry_run: args.dry_run,
        }
    }
}

#[derive(Debug, Serialize)]
struct InitPlan {
    project_config_dir: PathBuf,
    prompt_home: PathBuf,
    force: bool,
    dry_run: bool,
}

impl From<&InitArgs> for InitPlan {
    fn from(args: &InitArgs) -> Self {
        Self {
            project_config_dir: args.project_config_dir.clone(),
            prompt_home: args.prompt_home.clone(),
            force: args.force,
            dry_run: args.dry_run,
        }
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
