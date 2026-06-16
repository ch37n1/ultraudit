use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::util::{expand_tilde_path, resolve_path};

#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    pub prompt_pack_name: Option<String>,
    pub prompt_pack_version: Option<String>,
    pub prompt_pack_source: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub default_agent: Option<String>,
    pub disabled_optics: Vec<String>,
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
    })
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

#[derive(Debug, Clone, Default)]
struct SimpleToml {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl SimpleToml {
    fn read(path: &Path) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        Ok(Self::parse(&contents))
    }

    fn parse(contents: &str) -> Self {
        let mut parsed = Self::default();
        let mut section = String::new();

        for line in contents.lines() {
            let stripped = strip_comment(line);
            let line = stripped.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(raw_section) = line
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
            {
                section = raw_section.trim().to_owned();
                parsed.sections.entry(section.clone()).or_default();
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            parsed
                .sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().to_owned(), parse_value(value.trim()));
        }

        parsed
    }

    fn has_section(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    fn value(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|values| values.get(key))
            .map(String::as_str)
    }

    fn array(&self, section: &str, key: &str) -> Vec<String> {
        let Some(value) = self.value(section, key) else {
            return Vec::new();
        };

        let Some(inner) = value
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        else {
            return Vec::new();
        };

        inner
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(parse_value)
            .collect()
    }
}

fn strip_comment(line: &str) -> String {
    let mut in_quote = false;
    let mut previous = '\0';

    for (index, ch) in line.char_indices() {
        if ch == '"' && previous != '\\' {
            in_quote = !in_quote;
        }

        if ch == '#' && !in_quote {
            return line[..index].to_owned();
        }

        previous = ch;
    }

    line.to_owned()
}

fn parse_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\\\", "\\")
    } else {
        value.to_owned()
    }
}
