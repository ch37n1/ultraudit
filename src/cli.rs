use std::{fmt, path::PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

const LONG_HELP: &str = "\
EXAMPLES:
    make install
    uat init
    uat run --pack full
    uat run --pack production --domain auth
    uat run --lens security --lens correctness
    uat run --optic documentation-knowledge
    uat run --agent codex --prompt-home ~/.ultraudit
    uat run --jobs 4 --retries 1
    ULTRAUDIT_PATH=./for-test uat run --dry-run
    uat run --plan --pack default

COMPLETIONS:
    uat completions zsh > ~/.zfunc/_uat
    uat completions bash > ~/.local/share/bash-completion/completions/uat
";

#[derive(Debug, clap::Parser)]
#[command(
    name = "uat",
    version,
    about,
    author,
    propagate_version = true,
    arg_required_else_help = true,
    color = clap::ColorChoice::Auto,
    after_long_help = LONG_HELP
)]
pub struct Cli {
    #[command(flatten)]
    pub output: OutputArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Args)]
pub struct OutputArgs {
    /// Increase logging verbosity. Repeat for more detail: -v, -vv, -vvv.
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Control colored output.
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    pub color: ColorMode,

    /// Choose human-readable text or machine-readable JSON output.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Prepare and start an audit run.
    Run(RunArgs),

    /// Prepare project-local Ultraudit configuration.
    Init(InitArgs),

    /// Generate shell completions.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    /// Repository root to audit.
    #[arg(short = 'r', long = "repo", value_name = "DIR", default_value = ".")]
    pub repository: PathBuf,

    /// Project-local Ultraudit config file.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Directory where run artifacts will be written.
    #[arg(short = 'o', long, value_name = "DIR", default_value = ".audit-runs")]
    pub output_dir: PathBuf,

    /// User-level prompt/practice home.
    #[arg(
        long,
        value_name = "DIR",
        default_value = "~/.ultraudit",
        env = "ULTRAUDIT_PATH"
    )]
    pub prompt_home: PathBuf,

    /// Deprecated prompt pack compatibility name.
    #[arg(long, value_name = "NAME", default_value = "default")]
    pub pack_name: String,

    /// Prompt pack version.
    #[arg(long, value_name = "VERSION", default_value = "0.2.0")]
    pub pack_version: String,

    /// Named lens pack to run.
    #[arg(short = 'p', long, value_enum, default_value_t = LensPack::Default)]
    pub pack: LensPack,

    /// Run one or more specific core lenses instead of the whole pack.
    #[arg(short = 'l', long = "lens", value_enum, value_name = "LENS")]
    pub lenses: Vec<Lens>,

    /// Run one or more supplemental optics.
    #[arg(long = "optic", value_enum, value_name = "OPTIC")]
    pub optics: Vec<Optic>,

    /// Limit the run to one or more known domains.
    #[arg(short = 'd', long = "domain", value_name = "DOMAIN")]
    pub domains: Vec<String>,

    /// Agent runner configured for this audit.
    #[arg(short, long, value_name = "AGENT", default_value = "codex")]
    pub agent: String,

    /// Previous Ultraudit run directory to compare against. Repeat for multiple runs.
    #[arg(long = "previous-run", value_name = "DIR")]
    pub previous_runs: Vec<PathBuf>,

    /// Do not auto-discover previous runs from the output directory.
    #[arg(long)]
    pub no_previous_runs: bool,

    /// Continue the flow and preserve artifacts when an agent command fails.
    #[arg(long)]
    pub allow_agent_failures: bool,

    /// Maximum number of agent steps to run at once.
    #[arg(long, value_name = "N", default_value_t = 4, value_parser = parse_jobs)]
    pub jobs: usize,

    /// Retry a failed agent step before accepting failure.
    #[arg(long, value_name = "N", default_value_t = 1, value_parser = parse_retries)]
    pub retries: u8,

    /// Run the full audit flow with fake agent calls.
    #[arg(long)]
    pub dry_run: bool,

    /// Validate CLI input and print the resolved plan without writing files or starting agents.
    #[arg(long, conflicts_with = "dry_run")]
    pub plan: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// Project-local config directory to prepare.
    #[arg(long, value_name = "DIR", default_value = ".audit")]
    pub project_config_dir: PathBuf,

    /// User-level prompt/practice home.
    #[arg(
        long,
        value_name = "DIR",
        default_value = "~/.ultraudit",
        env = "ULTRAUDIT_PATH"
    )]
    pub prompt_home: PathBuf,

    /// Deprecated prompt pack compatibility name.
    #[arg(long, value_name = "NAME", default_value = "default")]
    pub pack_name: String,

    /// Prompt pack version to reference in project config.
    #[arg(long, value_name = "VERSION", default_value = "0.2.0")]
    pub pack_version: String,

    /// Overwrite existing starter files when init is implemented.
    #[arg(short, long)]
    pub force: bool,

    /// Validate paths and print the resolved plan without writing files.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum LensPack {
    Default,
    Production,
    ContractsAndData,
    Product,
    Full,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Lens {
    Architecture,
    CodeQuality,
    Security,
    Correctness,
    Testing,
    Reliability,
    Performance,
    Observability,
    Operations,
    ApiContracts,
    DataIntegrity,
    PrivacyCompliance,
    DependencySupplyChain,
    UxProduct,
    MlAi,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Optic {
    DocumentationKnowledge,
    NicePractices,
}

impl fmt::Display for LensPack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", value_name(self))
    }
}

impl fmt::Display for Lens {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", value_name(self))
    }
}

impl fmt::Display for Optic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", value_name(self))
    }
}

impl LensPack {
    pub fn as_str(self) -> &'static str {
        match self {
            LensPack::Default => "default",
            LensPack::Production => "production",
            LensPack::ContractsAndData => "contracts-and-data",
            LensPack::Product => "product",
            LensPack::Full => "full",
        }
    }

    pub fn lenses(self) -> &'static [Lens] {
        match self {
            LensPack::Default => Lens::all(),
            LensPack::Production => &[
                Lens::Reliability,
                Lens::Performance,
                Lens::Observability,
                Lens::Operations,
            ],
            LensPack::ContractsAndData => &[
                Lens::ApiContracts,
                Lens::DataIntegrity,
                Lens::PrivacyCompliance,
                Lens::DependencySupplyChain,
            ],
            LensPack::Product => &[Lens::UxProduct, Lens::MlAi],
            LensPack::Full => Lens::all(),
        }
    }
}

impl Lens {
    pub fn all() -> &'static [Lens] {
        &[
            Lens::Architecture,
            Lens::CodeQuality,
            Lens::Security,
            Lens::Correctness,
            Lens::Testing,
            Lens::Reliability,
            Lens::Performance,
            Lens::Observability,
            Lens::Operations,
            Lens::ApiContracts,
            Lens::DataIntegrity,
            Lens::PrivacyCompliance,
            Lens::DependencySupplyChain,
            Lens::UxProduct,
            Lens::MlAi,
        ]
    }

    pub fn cross_system_default() -> &'static [Lens] {
        &[
            Lens::Architecture,
            Lens::Security,
            Lens::Reliability,
            Lens::Operations,
            Lens::ApiContracts,
            Lens::DataIntegrity,
            Lens::PrivacyCompliance,
            Lens::MlAi,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lens::Architecture => "architecture",
            Lens::CodeQuality => "code-quality",
            Lens::Security => "security",
            Lens::Correctness => "correctness",
            Lens::Testing => "testing",
            Lens::Reliability => "reliability",
            Lens::Performance => "performance",
            Lens::Observability => "observability",
            Lens::Operations => "operations",
            Lens::ApiContracts => "api-contracts",
            Lens::DataIntegrity => "data-integrity",
            Lens::PrivacyCompliance => "privacy-compliance",
            Lens::DependencySupplyChain => "dependency-supply-chain",
            Lens::UxProduct => "ux-product",
            Lens::MlAi => "ml-ai",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|lens| lens.as_str() == value)
    }

    pub fn title(self) -> &'static str {
        match self {
            Lens::Architecture => "Architecture",
            Lens::CodeQuality => "Code Quality / Maintainability",
            Lens::Security => "Security",
            Lens::Correctness => "Correctness",
            Lens::Testing => "Testing",
            Lens::Reliability => "Reliability / Resilience",
            Lens::Performance => "Performance / Scalability",
            Lens::Observability => "Observability",
            Lens::Operations => "Operations / Deployment",
            Lens::ApiContracts => "API / Contract Design",
            Lens::DataIntegrity => "Data Integrity",
            Lens::PrivacyCompliance => "Privacy / Compliance",
            Lens::DependencySupplyChain => "Dependency / Supply Chain",
            Lens::UxProduct => "UX / Product Behavior",
            Lens::MlAi => "ML / AI Systems Review",
        }
    }
}

impl Optic {
    pub fn all_default() -> &'static [Optic] {
        &[Optic::DocumentationKnowledge, Optic::NicePractices]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Optic::DocumentationKnowledge => "documentation-knowledge",
            Optic::NicePractices => "nice-practices",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        Self::all_default()
            .iter()
            .copied()
            .find(|optic| optic.as_str() == value)
    }

    pub fn title(self) -> &'static str {
        match self {
            Optic::DocumentationKnowledge => "Documentation / Knowledge",
            Optic::NicePractices => "Nice Practices",
        }
    }
}

fn value_name<T: ValueEnum>(value: &T) -> String {
    value
        .to_possible_value()
        .expect("value enum variants must have names")
        .get_name()
        .to_owned()
}

fn parse_jobs(value: &str) -> Result<usize, String> {
    let jobs = value
        .parse::<usize>()
        .map_err(|_| format!("`{value}` is not a valid job count"))?;
    if jobs == 0 {
        return Err("job count must be at least 1".to_owned());
    }
    Ok(jobs)
}

fn parse_retries(value: &str) -> Result<u8, String> {
    let retries = value
        .parse::<u8>()
        .map_err(|_| format!("`{value}` is not a valid retry count"))?;
    if retries > 3 {
        return Err("retry count must be between 0 and 3".to_owned());
    }
    Ok(retries)
}
