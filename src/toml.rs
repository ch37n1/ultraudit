use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub(crate) struct SimpleToml {
    pub(crate) sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl SimpleToml {
    pub(crate) fn read(path: &Path) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        Ok(Self::parse(&contents))
    }

    pub(crate) fn parse(contents: &str) -> Self {
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

    pub(crate) fn has_section(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    pub(crate) fn value(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|values| values.get(key))
            .map(String::as_str)
    }

    pub(crate) fn array(&self, section: &str, key: &str) -> Vec<String> {
        let Some(value) = self.value(section, key) else {
            return Vec::new();
        };

        parse_array(value)
    }
}

pub(crate) fn parse_array(value: &str) -> Vec<String> {
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
