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
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    run_id_from_unix_time(now.as_secs(), now.subsec_nanos(), std::process::id())
}

fn run_id_from_unix_time(seconds: u64, nanos: u32, process_id: u32) -> String {
    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_date_from_epoch_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}.{nanos:09}Z-run-{process_id}"
    )
}

fn civil_date_from_epoch_days(days_since_epoch: u64) -> (i64, u64, u64) {
    // Convert Unix epoch days to a proleptic Gregorian date using 400-year cycles.
    let days = days_since_epoch as i64 + 719_468;
    let era = days / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_param = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_param + 2) / 5 + 1;
    let month = month_param + if month_param < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };

    (year, month as u64, day as u64)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_id_from_unix_time_formats_epoch_as_readable_utc_timestamp() {
        assert_eq!(
            run_id_from_unix_time(0, 0, 42),
            "1970-01-01T00-00-00.000000000Z-run-42"
        );
    }

    #[test]
    fn run_id_from_unix_time_formats_leap_day() {
        assert_eq!(
            run_id_from_unix_time(951_782_400, 123_456_789, 7),
            "2000-02-29T00-00-00.123456789Z-run-7"
        );
    }
}
