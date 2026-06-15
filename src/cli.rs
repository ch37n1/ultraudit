use std::{fmt, path::PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

const LONG_HELP: &str = "\
EXAMPLES:
    ultraudit run --pack full
    ultraudit run --pack production --domain auth
    ultraudit run --lens security --lens correctness
    ultraudit run --optic documentation-knowledge
    ultraudit run --agent codex --prompt-home ~/.ultraudit
    ultraudit run --dry-run --pack default

COMPLETIONS:
    ultraudit completions zsh > ~/.zfunc/_ultraudit
    ultraudit completions bash > ~/.local/share/bash-completion/completions/ultraudit
";

#[derive(Debug, clap::Parser)]
#[command(
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
    #[arg(long, value_name = "DIR", default_value = "~/.ultraudit")]
    pub prompt_home: PathBuf,

    /// Prompt pack name.
    #[arg(long, value_name = "NAME", default_value = "ultraudit-default")]
    pub pack_name: String,

    /// Prompt pack version.
    #[arg(long, value_name = "VERSION", default_value = "0.1.0")]
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

    /// Validate CLI input and print the resolved plan without starting agents.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// Project-local config directory to prepare.
    #[arg(long, value_name = "DIR", default_value = ".audit")]
    pub project_config_dir: PathBuf,

    /// User-level prompt/practice home.
    #[arg(long, value_name = "DIR", default_value = "~/.ultraudit")]
    pub prompt_home: PathBuf,

    /// Prompt pack name to seed.
    #[arg(long, value_name = "NAME", default_value = "ultraudit-default")]
    pub pack_name: String,

    /// Prompt pack version to seed.
    #[arg(long, value_name = "VERSION", default_value = "0.1.0")]
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
            LensPack::Default => &[
                Lens::Architecture,
                Lens::CodeQuality,
                Lens::Security,
                Lens::Correctness,
                Lens::Testing,
            ],
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
