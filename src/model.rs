use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditPlan {
    pub repository: PathBuf,
    pub output_dir: PathBuf,
    pub config: Option<PathBuf>,
    pub prompt_home: PathBuf,
    pub pack_name: String,
    pub pack_version: String,
    pub pack_source: PathBuf,
    pub pack: String,
    pub agent: String,
    pub model: Option<String>,
    pub lenses: Vec<String>,
    pub optics: Vec<String>,
    pub domains: Vec<String>,
    pub previous_runs: Vec<PathBuf>,
    pub dry_run: bool,
    pub allow_agent_failures: bool,
    pub jobs: usize,
    pub retries: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitPlan {
    pub project_config_dir: PathBuf,
    pub prompt_home: PathBuf,
    pub pack_name: String,
    pub pack_version: String,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunManifest {
    pub run_id: String,
    pub repository: PathBuf,
    pub output_dir: PathBuf,
    pub run_dir: PathBuf,
    pub config: Option<PathBuf>,
    pub prompt_pack: PromptPackManifest,
    pub agent: String,
    pub model: Option<String>,
    pub selected_pack: String,
    pub selected_lenses: Vec<String>,
    pub selected_optics: Vec<String>,
    pub requested_domains: Vec<String>,
    pub previous_runs: Vec<PathBuf>,
    pub dry_run: bool,
    pub allow_agent_failures: bool,
    pub jobs: usize,
    pub retries: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptPackManifest {
    pub name: String,
    pub version: String,
    pub source: PathBuf,
    pub snapshot: PathBuf,
    pub content_fingerprint: String,
    pub file_count: usize,
    pub byte_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryContext {
    pub root: PathBuf,
    pub git: GitContext,
    pub languages: Vec<String>,
    pub package_files: Vec<String>,
    pub key_files: Vec<String>,
    pub directories: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub dirty: bool,
    pub status_short: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Domain {
    pub domain_id: String,
    pub name: String,
    pub description: String,
    pub key_paths: Vec<String>,
    pub neighboring_domains: Vec<String>,
    pub external_dependencies: Vec<String>,
    pub risk_areas: Vec<String>,
    pub recommended_lenses: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainMap {
    pub domains: Vec<Domain>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInvocationManifest {
    pub step_id: String,
    pub role: String,
    pub kind: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub prompt_transport: String,
    pub timeout_seconds: u64,
    pub prompt_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub report_path: PathBuf,
    pub findings_path: PathBuf,
    pub notes_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentExit {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStepRecord {
    pub invocation: AgentInvocationManifest,
    pub exit: AgentExit,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub run_dir: PathBuf,
    pub final_report: PathBuf,
    pub domains: Vec<String>,
    pub lenses: Vec<String>,
    pub optics: Vec<String>,
    pub model: Option<String>,
    pub dry_run: bool,
    pub jobs: usize,
    pub retries: u8,
    pub steps: Vec<AgentStepRecord>,
}
