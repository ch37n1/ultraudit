use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    toml::SimpleToml,
    util::{expand_tilde_path, resolve_path},
};

const DEFAULT_CODEX_MODEL: &str = "gpt-5.5";

#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    pub prompt_pack_name: Option<String>,
    pub prompt_pack_version: Option<String>,
    pub prompt_pack_source: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub default_agent: Option<String>,
    pub disabled_optics: Vec<String>,
    pub artifact_retention_days: Option<u64>,
    pub prompt_max_chars: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub default_model: Option<String>,
    pub agent_models: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub kind: AgentKind,
    pub binary: PathBuf,
    pub mode: String,
    pub model: Option<String>,
    pub ignore_user_config: bool,
    pub prompt_transport: PromptTransport,
    pub approval_policy: String,
    pub sandbox: String,
    pub timeout_seconds: u64,
    pub shell: String,
    pub command: Option<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AgentKind {
    CodexCli,
    ShellTemplate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PromptTransport {
    Stdin,
    PromptFile,
}

impl AgentConfig {
    pub fn default_codex() -> Self {
        Self {
            name: "codex".to_owned(),
            kind: AgentKind::CodexCli,
            binary: PathBuf::from("codex"),
            mode: "exec".to_owned(),
            model: None,
            ignore_user_config: true,
            prompt_transport: PromptTransport::Stdin,
            approval_policy: "never".to_owned(),
            sandbox: "workspace-write".to_owned(),
            timeout_seconds: 7200,
            shell: "sh".to_owned(),
            command: None,
            env: BTreeMap::new(),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            AgentKind::CodexCli => "codex-cli",
            AgentKind::ShellTemplate => "shell-template",
        }
    }

    pub fn prompt_transport_name(&self) -> &'static str {
        match self.prompt_transport {
            PromptTransport::Stdin => "stdin",
            PromptTransport::PromptFile => "prompt-file",
        }
    }
}

pub fn find_project_config(explicit: Option<&Path>, repository: &Path) -> Option<PathBuf> {
    explicit.map(PathBuf::from).or_else(|| {
        let candidate = repository.join(".audit/config.toml");
        candidate.exists().then_some(candidate)
    })
}

pub fn load_project_config(path: Option<&Path>) -> Result<ProjectConfig> {
    let Some(path) = path else {
        return Ok(ProjectConfig::default());
    };

    if !path.exists() {
        return Ok(ProjectConfig::default());
    }

    let parsed = SimpleToml::read(path)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));

    Ok(ProjectConfig {
        prompt_pack_name: parsed.value("prompt_pack", "name").map(ToOwned::to_owned),
        prompt_pack_version: parsed
            .value("prompt_pack", "version")
            .map(ToOwned::to_owned),
        prompt_pack_source: parsed
            .value("prompt_pack", "source")
            .map(PathBuf::from)
            .map(|path| resolve_path(&path, base)),
        output_dir: parsed
            .value("run", "output_dir")
            .map(PathBuf::from)
            .map(|path| expand_tilde_path(&path)),
        default_agent: parsed.value("run", "agent").map(ToOwned::to_owned),
        disabled_optics: parsed.array("run", "disabled_optics"),
        artifact_retention_days: parsed
            .value("artifacts", "retention_days")
            .filter(|value| !value.is_empty())
            .map(str::parse)
            .transpose()
            .context("parse artifacts.retention_days")?,
        prompt_max_chars: parsed
            .value("prompt", "max_chars")
            .filter(|value| !value.is_empty())
            .map(str::parse)
            .transpose()
            .context("parse prompt.max_chars")?,
    })
}

pub fn load_model_config(path: Option<&Path>) -> Result<ModelConfig> {
    let Some(path) = path else {
        return Ok(ModelConfig::default());
    };

    if !path.exists() {
        return Ok(ModelConfig::default());
    }

    let parsed = SimpleToml::read(path)?;
    let mut config = ModelConfig::default();
    let parsed_agent_models = parse_agent_models(&parsed);
    if let Some(default_model) = parsed
        .value("models", "default")
        .filter(|model| !model.is_empty())
    {
        config.default_model = Some(default_model.to_owned());
        if !parsed_agent_models.contains_key("codex") {
            config.agent_models.remove("codex");
        }
    }
    config.agent_models.extend(parsed_agent_models);
    Ok(config)
}

pub fn load_agent_config(
    name: &str,
    project_config: Option<&Path>,
    repository: &Path,
) -> Result<AgentConfig> {
    let agent_file = project_config
        .and_then(Path::parent)
        .map(|dir| dir.join("agents").join(format!("{name}.toml")))
        .or_else(|| {
            Some(
                repository
                    .join(".audit/agents")
                    .join(format!("{name}.toml")),
            )
        });

    if let Some(path) = agent_file {
        if path.exists() {
            return agent_from_file(name, &path);
        }
    }

    if let Some(path) = project_config {
        if path.exists() {
            let parsed = SimpleToml::read(path)?;
            let section = format!("agents.{name}");
            if parsed.has_section(&section) {
                return agent_from_toml(
                    name,
                    &parsed,
                    &section,
                    path.parent().unwrap_or(repository),
                );
            }
        }
    }

    if name == "codex" {
        Ok(AgentConfig::default_codex())
    } else {
        bail!("agent `{name}` is not configured in .audit/agents/{name}.toml")
    }
}

pub fn load_effective_agent_config(
    name: &str,
    project_config: Option<&Path>,
    repository: &Path,
) -> Result<AgentConfig> {
    let mut config = load_agent_config(name, project_config, repository)?;
    apply_model_config(&mut config, project_config, repository)?;
    Ok(config)
}

pub fn resolve_agent_model(
    name: &str,
    project_config: Option<&Path>,
    repository: &Path,
    allow_missing_agent: bool,
) -> Result<Option<String>> {
    let model_config = load_model_config(project_config)?;
    let model = model_config.model_for_agent(name).map(ToOwned::to_owned);

    match load_agent_config(name, project_config, repository) {
        Ok(agent) => Ok(agent.model.or(model)),
        Err(error)
            if allow_missing_agent
                && agent_config_is_missing(name, project_config, repository)? =>
        {
            Ok(model)
        }
        Err(error) => Err(error),
    }
}

impl ModelConfig {
    fn model_for_agent(&self, name: &str) -> Option<&str> {
        self.agent_models
            .get(name)
            .map(String::as_str)
            .or(self.default_model.as_deref())
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: None,
            agent_models: BTreeMap::from([("codex".to_owned(), DEFAULT_CODEX_MODEL.to_owned())]),
        }
    }
}

fn apply_model_config(
    config: &mut AgentConfig,
    project_config: Option<&Path>,
    _repository: &Path,
) -> Result<()> {
    if config.model.is_some() {
        return Ok(());
    }

    let model_config = load_model_config(project_config)?;
    if let Some(model) = model_config.model_for_agent(&config.name) {
        config.model = Some(model.to_owned());
    }
    Ok(())
}

fn parse_agent_models(parsed: &SimpleToml) -> BTreeMap<String, String> {
    let mut models = BTreeMap::new();

    if let Some(values) = parsed.sections.get("models") {
        for (key, value) in values {
            if key != "default" && !value.is_empty() {
                models.insert(key.clone(), value.clone());
            }
        }
    }

    for (section, values) in &parsed.sections {
        let Some(agent) = section.strip_prefix("agents.") else {
            continue;
        };
        if let Some(model) = values.get("model").filter(|model| !model.is_empty()) {
            models.insert(agent.to_owned(), model.clone());
        }
    }

    models
}

fn agent_config_is_missing(
    name: &str,
    project_config: Option<&Path>,
    repository: &Path,
) -> Result<bool> {
    if name == "codex" {
        return Ok(false);
    }

    if agent_config_file(name, project_config, repository).exists() {
        return Ok(false);
    }

    if let Some(path) = project_config.filter(|path| path.exists()) {
        let parsed = SimpleToml::read(path)?;
        if parsed.has_section(&format!("agents.{name}")) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn agent_from_file(name: &str, path: &Path) -> Result<AgentConfig> {
    let parsed = SimpleToml::read(path)?;
    agent_from_toml(
        name,
        &parsed,
        "",
        path.parent().unwrap_or_else(|| Path::new(".")),
    )
}

fn agent_from_toml(
    name: &str,
    parsed: &SimpleToml,
    section: &str,
    base: &Path,
) -> Result<AgentConfig> {
    let mut config = if name == "codex" {
        AgentConfig::default_codex()
    } else {
        AgentConfig {
            name: name.to_owned(),
            kind: AgentKind::ShellTemplate,
            binary: PathBuf::from("sh"),
            mode: String::new(),
            model: None,
            ignore_user_config: true,
            prompt_transport: PromptTransport::Stdin,
            approval_policy: "never".to_owned(),
            sandbox: "workspace-write".to_owned(),
            timeout_seconds: 7200,
            shell: "sh".to_owned(),
            command: None,
            env: BTreeMap::new(),
        }
    };

    config.name = name.to_owned();

    if let Some(kind) = parsed.value(section, "kind") {
        config.kind = parse_agent_kind(kind)?;
    }
    if let Some(binary) = parsed.value(section, "binary") {
        config.binary = command_or_path(binary, base);
    }
    if let Some(mode) = parsed.value(section, "mode") {
        config.mode = mode.to_owned();
    }
    if let Some(model) = parsed
        .value(section, "model")
        .filter(|model| !model.is_empty())
    {
        config.model = Some(model.to_owned());
    }
    if let Some(ignore_user_config) = parsed.value(section, "ignore_user_config") {
        config.ignore_user_config = parse_bool(ignore_user_config)
            .with_context(|| format!("parse ignore_user_config for agent `{name}`"))?;
    }
    if let Some(prompt_transport) = parsed.value(section, "prompt_transport") {
        config.prompt_transport = parse_prompt_transport(prompt_transport)?;
    }
    if let Some(approval_policy) = parsed.value(section, "approval_policy") {
        config.approval_policy = approval_policy.to_owned();
    }
    if let Some(sandbox) = parsed.value(section, "sandbox") {
        config.sandbox = sandbox.to_owned();
    }
    if let Some(timeout_seconds) = parsed.value(section, "timeout_seconds") {
        config.timeout_seconds = timeout_seconds
            .parse()
            .with_context(|| format!("parse timeout_seconds for agent `{name}`"))?;
    }
    if let Some(shell) = parsed.value(section, "shell") {
        config.shell = shell.to_owned();
    }
    if let Some(command) = parsed.value(section, "command") {
        config.command = Some(command.to_owned());
    }

    if matches!(config.kind, AgentKind::ShellTemplate) && config.command.is_none() {
        bail!("shell-template agent `{name}` must define command")
    }

    Ok(config)
}

fn agent_config_file(name: &str, project_config: Option<&Path>, repository: &Path) -> PathBuf {
    project_config
        .and_then(Path::parent)
        .map(|dir| dir.join("agents").join(format!("{name}.toml")))
        .unwrap_or_else(|| {
            repository
                .join(".audit/agents")
                .join(format!("{name}.toml"))
        })
}

fn command_or_path(value: &str, base: &Path) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() || value.contains('/') {
        resolve_path(&path, base)
    } else {
        path
    }
}

fn parse_agent_kind(value: &str) -> Result<AgentKind> {
    match value {
        "codex-cli" => Ok(AgentKind::CodexCli),
        "shell-template" => Ok(AgentKind::ShellTemplate),
        other => bail!("unsupported agent kind `{other}`"),
    }
}

fn parse_prompt_transport(value: &str) -> Result<PromptTransport> {
    match value {
        "stdin" => Ok(PromptTransport::Stdin),
        "prompt-file" => Ok(PromptTransport::PromptFile),
        other => bail!("unsupported prompt transport `{other}`"),
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        other => bail!("expected `true` or `false`, got `{other}`"),
    }
}
