use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::Serialize;

pub fn expand_tilde_path(path: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };

    if raw == "~" {
        return home_dir().unwrap_or_else(|| path.to_path_buf());
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    path.to_path_buf()
}

pub fn resolve_path(path: &Path, base: &Path) -> PathBuf {
    let expanded = expand_tilde_path(path);
    if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    }
}

pub fn now_run_id() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    format!("run-{seconds}-{}", std::process::id())
}

pub fn sanitize_id(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }

    let trimmed = output.trim_matches('-');
    if trimmed.is_empty() {
        "item".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub fn write_text(path: &Path, contents: impl AsRef<str>) -> Result<()> {
    ensure_parent(path)?;
    fs::write(path, contents.as_ref()).with_context(|| format!("write {}", path.display()))
}

pub fn write_text_if_absent(path: &Path, contents: impl AsRef<str>, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }

    write_text(path, contents)
}

pub fn write_json_yaml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut contents = serde_json::to_string_pretty(value)
        .with_context(|| format!("serialize {}", path.display()))?;
    contents.push('\n');
    write_text(path, contents)
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    Ok(())
}

pub fn read_optional(path: &Path) -> Result<Option<String>> {
    if path.exists() {
        fs::read_to_string(path)
            .map(Some)
            .with_context(|| format!("read {}", path.display()))
    } else {
        Ok(None)
    }
}

pub fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        return Ok(());
    }

    fs::create_dir_all(to).with_context(|| format!("create {}", to.display()))?;

    for entry in fs::read_dir(from).with_context(|| format!("read {}", from.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", from.display()))?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        let metadata = entry
            .metadata()
            .with_context(|| format!("metadata {}", source.display()))?;

        if metadata.is_dir() {
            copy_dir_recursive(&source, &target)?;
        } else if metadata.is_file() {
            ensure_parent(&target)?;
            fs::copy(&source, &target)
                .with_context(|| format!("copy {} to {}", source.display(), target.display()))?;
        }
    }

    Ok(())
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub fn relative_display(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .map(display_path)
        .unwrap_or_else(|_| display_path(path))
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("\n[truncated]\n");
    truncated
}

pub fn command_display(program: &Path, args: &[String]) -> String {
    std::iter::once(shell_escape_os(program.as_os_str()))
        .chain(args.iter().map(|arg| shell_escape(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':' | '='))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub fn shell_escape_os(value: &OsStr) -> String {
    shell_escape(&value.to_string_lossy())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
