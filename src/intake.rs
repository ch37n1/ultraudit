use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};

use crate::{
    cli::Lens,
    model::{Domain, GitContext, RepositoryContext},
    util::{relative_display, sanitize_id, write_json_yaml, write_text},
};

const WALK_LIMIT: usize = 900;

pub fn collect_repository_context(repository: &Path) -> Result<RepositoryContext> {
    let root = repository
        .canonicalize()
        .with_context(|| format!("canonicalize {}", repository.display()))?;
    let git = collect_git_context(&root);
    let mut key_files = Vec::new();
    let mut package_files = Vec::new();
    let mut directories = BTreeSet::new();
    let mut language_counts: BTreeMap<&'static str, usize> = BTreeMap::new();

    for path in walk_repository(&root)? {
        let relative = relative_display(&path, &root);
        if path.is_dir() {
            directories.insert(relative);
            continue;
        }

        if is_package_file(&path) {
            package_files.push(relative.clone());
            key_files.push(relative.clone());
        } else if is_key_file(&path) {
            key_files.push(relative.clone());
        }

        if let Some(language) = language_for_path(&path) {
            *language_counts.entry(language).or_default() += 1;
        }
    }

    let mut languages = language_counts
        .into_iter()
        .map(|(language, count)| format!("{language} ({count})"))
        .collect::<Vec<_>>();
    languages.sort();
    package_files.sort();
    key_files.sort();
    let directories = directories.into_iter().collect();

    Ok(RepositoryContext {
        root,
        git,
        languages,
        package_files,
        key_files,
        directories,
    })
}

pub fn infer_domains(context: &RepositoryContext, requested: &[String]) -> Vec<Domain> {
    if !requested.is_empty() {
        return requested
            .iter()
            .map(|name| Domain {
                domain_id: sanitize_id(name),
                name: name.to_owned(),
                description: format!("User-requested review domain `{name}`."),
                key_paths: context
                    .directories
                    .iter()
                    .filter(|path| path.contains(name))
                    .take(12)
                    .cloned()
                    .collect(),
                neighboring_domains: Vec::new(),
                external_dependencies: context.package_files.clone(),
                risk_areas: default_risk_areas(),
                recommended_lenses: default_recommended_lenses(),
            })
            .collect();
    }

    let mut candidates = Vec::new();
    for directory in &context.directories {
        if directory.contains('/') {
            continue;
        }
        if matches!(
            directory.as_str(),
            "." | ".audit" | ".audit-runs" | ".git" | ".local" | "target"
        ) {
            continue;
        }
        candidates.push(directory.clone());
    }

    if candidates.is_empty() {
        candidates.push("root".to_owned());
    }

    candidates
        .into_iter()
        .take(12)
        .map(|name| {
            let domain_id = sanitize_id(&name);
            let key_paths = if name == "root" {
                context.key_files.iter().take(20).cloned().collect()
            } else {
                context
                    .directories
                    .iter()
                    .chain(context.key_files.iter())
                    .filter(|path| path == &&name || path.starts_with(&format!("{name}/")))
                    .take(24)
                    .cloned()
                    .collect()
            };

            Domain {
                domain_id,
                name: title_from_id(&name),
                description: format!("Repository area rooted at `{name}`."),
                key_paths,
                neighboring_domains: Vec::new(),
                external_dependencies: context.package_files.clone(),
                risk_areas: default_risk_areas(),
                recommended_lenses: default_recommended_lenses(),
            }
        })
        .collect()
}

pub fn write_repository_artifacts(
    run_dir: &Path,
    context: &RepositoryContext,
) -> Result<(PathBuf, PathBuf)> {
    let md_path = run_dir.join("repository.md");
    let yaml_path = run_dir.join("repository.yaml");
    write_text(&md_path, repository_markdown(context))?;
    write_json_yaml(&yaml_path, context)?;
    Ok((md_path, yaml_path))
}

pub fn repository_markdown(context: &RepositoryContext) -> String {
    let mut output = String::new();
    output.push_str("# Repository Intake\n\n");
    output.push_str(&format!("Root: `{}`\n\n", context.root.display()));
    output.push_str("## Git\n\n");
    output.push_str(&format!(
        "- branch: {}\n",
        context.git.branch.as_deref().unwrap_or("unknown")
    ));
    output.push_str(&format!(
        "- commit: {}\n",
        context.git.commit.as_deref().unwrap_or("unknown")
    ));
    output.push_str(&format!("- dirty: {}\n\n", context.git.dirty));
    push_list(&mut output, "Languages", &context.languages);
    push_list(&mut output, "Package / Build Files", &context.package_files);
    push_list(&mut output, "Key Files", &context.key_files);
    push_list(&mut output, "Top Directories", &context.directories);
    output
}

pub fn domain_markdown(domain: &Domain) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {}\n\n", domain.name));
    output.push_str(&format!("- id: `{}`\n", domain.domain_id));
    output.push_str(&format!("- responsibility: {}\n\n", domain.description));
    push_list(&mut output, "Key Paths", &domain.key_paths);
    push_list(
        &mut output,
        "External Dependencies",
        &domain.external_dependencies,
    );
    push_list(&mut output, "Risk Areas", &domain.risk_areas);
    push_list(
        &mut output,
        "Recommended Lenses",
        &domain.recommended_lenses,
    );
    output
}

pub fn domain_map_markdown(domains: &[Domain]) -> String {
    let mut output = String::new();
    output.push_str("# Domain Map\n\n");
    for domain in domains {
        output.push_str(&format!("## {}\n\n", domain.name));
        output.push_str(&format!("- id: `{}`\n", domain.domain_id));
        output.push_str(&format!("- responsibility: {}\n", domain.description));
        if !domain.key_paths.is_empty() {
            output.push_str("- key paths: ");
            output.push_str(&domain.key_paths.join(", "));
            output.push('\n');
        }
        output.push('\n');
    }
    output
}

fn collect_git_context(root: &Path) -> GitContext {
    let branch = git_output(root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let commit = git_output(root, &["rev-parse", "HEAD"]);
    let status_short = git_output(root, &["status", "--short"]).unwrap_or_default();

    GitContext {
        branch,
        commit,
        dirty: !status_short.trim().is_empty(),
        status_short,
    }
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn walk_repository(root: &Path) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    let mut queue = VecDeque::from([root.to_path_buf()]);

    while let Some(directory) = queue.pop_front() {
        if output.len() >= WALK_LIMIT {
            break;
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("read directory {}", directory.display()))?
        {
            entries.push(entry.with_context(|| format!("read entry in {}", directory.display()))?);
        }
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            if should_skip(root, &path) {
                continue;
            }

            output.push(path.clone());

            if path.is_dir() {
                queue.push_back(path);
            }

            if output.len() >= WALK_LIMIT {
                break;
            }
        }
    }

    Ok(output)
}

fn should_skip(root: &Path, path: &Path) -> bool {
    let relative = relative_display(path, root);
    let first = relative.split('/').next().unwrap_or_default();
    matches!(
        first,
        ".git" | ".mypy_cache" | ".audit-runs" | "target" | "node_modules" | "dist" | "build"
    )
}

fn is_package_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    matches!(
        name,
        "Cargo.toml"
            | "Cargo.lock"
            | "package.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "package-lock.json"
            | "pyproject.toml"
            | "requirements.txt"
            | "uv.lock"
            | "poetry.lock"
            | "go.mod"
            | "go.sum"
            | "pom.xml"
            | "build.gradle"
            | "settings.gradle"
            | "Dockerfile"
            | "docker-compose.yml"
    )
}

fn is_key_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    matches!(
        name,
        "README.md"
            | "Makefile"
            | ".gitignore"
            | "tsconfig.json"
            | "vite.config.ts"
            | "next.config.js"
            | "next.config.ts"
            | "pytest.ini"
            | "mypy.ini"
            | "ruff.toml"
    )
}

fn language_for_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Some("Rust"),
        Some("py") => Some("Python"),
        Some("ts" | "tsx") => Some("TypeScript"),
        Some("js" | "jsx" | "mjs" | "cjs") => Some("JavaScript"),
        Some("html") => Some("HTML"),
        Some("css" | "scss" | "sass") => Some("CSS"),
        Some("swift") => Some("Swift"),
        Some("kt" | "kts") => Some("Kotlin"),
        Some("go") => Some("Go"),
        Some("java") => Some("Java"),
        Some("md" | "mdx") => Some("Markdown"),
        Some("toml") => Some("TOML"),
        Some("yaml" | "yml") => Some("YAML"),
        Some("json") => Some("JSON"),
        _ => None,
    }
}

fn default_risk_areas() -> Vec<String> {
    vec![
        "boundary ownership".to_owned(),
        "security and privacy exposure".to_owned(),
        "correctness edge cases".to_owned(),
        "test coverage".to_owned(),
    ]
}

fn default_recommended_lenses() -> Vec<String> {
    Lens::all()
        .iter()
        .take(5)
        .map(|lens| lens.as_str().to_owned())
        .collect()
}

fn title_from_id(value: &str) -> String {
    value
        .split(['-', '_', '/'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.collect::<String>())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_list(output: &mut String, title: &str, values: &[String]) {
    output.push_str(&format!("## {title}\n\n"));
    if values.is_empty() {
        output.push_str("- none detected\n\n");
        return;
    }

    for value in values.iter().take(80) {
        output.push_str(&format!("- `{value}`\n"));
    }
    if values.len() > 80 {
        output.push_str("- ...\n");
    }
    output.push('\n');
}
