use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    cli::{Lens, Optic},
    util::{copy_dir_recursive, read_optional, write_text_if_absent},
};

#[derive(Debug, Clone)]
pub struct Pack {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
}

impl Pack {
    pub fn prompt(&self, name: &str) -> PathBuf {
        self.root.join("prompts").join(format!("{name}.md"))
    }

    pub fn lens_prompt(&self, lens: Lens) -> PathBuf {
        self.root
            .join("lenses")
            .join(lens.as_str())
            .join("prompt.md")
    }

    pub fn lens_practices(&self, lens: Lens) -> PathBuf {
        self.root
            .join("lenses")
            .join(lens.as_str())
            .join("practices.md")
    }

    pub fn optic_prompt(&self, optic: Optic) -> PathBuf {
        self.root
            .join("optics")
            .join(optic.as_str())
            .join("prompt.md")
    }

    pub fn optic_practices(&self, optic: Optic) -> PathBuf {
        self.root
            .join("optics")
            .join(optic.as_str())
            .join("practices.md")
    }
}

pub fn pack_root(prompt_home: &Path, name: &str, version: &str) -> PathBuf {
    prompt_home
        .join("packs")
        .join(name)
        .join("versions")
        .join(version)
}

pub fn ensure_default_pack(prompt_home: &Path, name: &str, version: &str) -> Result<Pack> {
    let root = pack_root(prompt_home, name, version);
    if !root.exists() && name == "ultraudit-default" {
        seed_default_pack(prompt_home, name, version, false)?;
    }

    Ok(Pack {
        name: name.to_owned(),
        version: version.to_owned(),
        root,
    })
}

pub fn seed_default_pack(
    prompt_home: &Path,
    name: &str,
    version: &str,
    force: bool,
) -> Result<PathBuf> {
    let root = pack_root(prompt_home, name, version);
    fs::create_dir_all(&root).with_context(|| format!("create {}", root.display()))?;

    seed_pack_manifest(&root, name, version, force)?;
    seed_prompts(&root, force)?;
    seed_lenses(&root, force)?;
    seed_optics(&root, force)?;
    seed_overlays(&root, force)?;
    seed_integration(&root, force)?;
    fs::create_dir_all(root.join("suggestions/pending"))
        .with_context(|| format!("create suggestions in {}", root.display()))?;

    Ok(root)
}

pub fn snapshot_pack(pack: &Pack, run_dir: &Path) -> Result<PathBuf> {
    let snapshot = run_dir.join("prompt-pack");
    copy_dir_recursive(&pack.root, &snapshot)?;
    Ok(snapshot)
}

pub fn render_template(template: &str, values: &BTreeMap<&str, String>) -> String {
    let mut rendered = template.to_owned();
    for (key, value) in values {
        rendered = rendered.replace(&format!("{{{{ {key} }}}}"), value);
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
    }
    rendered
}

pub fn read_pack_text(path: &Path) -> Result<String> {
    read_optional(path).map(|value| value.unwrap_or_else(|| missing_pack_file(path)))
}

fn seed_pack_manifest(root: &Path, name: &str, version: &str, force: bool) -> Result<()> {
    let contents = format!(
        r#"schema_version = "1"
name = "{name}"
version = "{version}"

[packs]
default = ["architecture", "code-quality", "security", "correctness", "testing"]
production = ["reliability", "performance", "observability", "operations"]
contracts-and-data = ["api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain"]
product = ["ux-product", "ml-ai"]
full = ["architecture", "code-quality", "security", "correctness", "testing", "reliability", "performance", "observability", "operations", "api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain", "ux-product", "ml-ai"]

[optics]
default = ["documentation-knowledge", "nice-practices"]
"#
    );

    write_text_if_absent(&root.join("pack.toml"), contents, force)
}

fn seed_prompts(root: &Path, force: bool) -> Result<()> {
    let prompts = root.join("prompts");
    fs::create_dir_all(&prompts).with_context(|| format!("create {}", prompts.display()))?;

    write_text_if_absent(
        &prompts.join("domain-discovery.md"),
        domain_discovery_prompt(),
        force,
    )?;
    write_text_if_absent(
        &prompts.join("cross-domain.md"),
        cross_domain_prompt(),
        force,
    )?;

    copy_research_or_write(
        "prompts/base-reviewer-guidance.md",
        &prompts.join("base-reviewer.md"),
        base_reviewer_fallback(),
        force,
    )?;
    copy_research_or_write(
        "prompts/lens-review-template.md",
        &prompts.join("lens-review.md"),
        lens_review_fallback(),
        force,
    )?;
    copy_research_or_write(
        "prompts/domain-synthesis-template.md",
        &prompts.join("domain-synthesis.md"),
        domain_synthesis_fallback(),
        force,
    )?;
    copy_research_or_write(
        "prompts/system-synthesis-template.md",
        &prompts.join("system-synthesis.md"),
        system_synthesis_fallback(),
        force,
    )?;
    copy_research_or_write(
        "prompts/previous-runs-comparison-template.md",
        &prompts.join("previous-runs-comparison.md"),
        previous_runs_fallback(),
        force,
    )?;
    copy_research_or_write(
        "prompts/final-editor-template.md",
        &prompts.join("final-editor.md"),
        final_editor_fallback(),
        force,
    )?;

    Ok(())
}

fn seed_lenses(root: &Path, force: bool) -> Result<()> {
    for lens in Lens::all() {
        let lens_dir = root.join("lenses").join(lens.as_str());
        fs::create_dir_all(&lens_dir).with_context(|| format!("create {}", lens_dir.display()))?;
        write_text_if_absent(
            &lens_dir.join("lens.toml"),
            format!("id = \"{}\"\nname = \"{}\"\n", lens.as_str(), lens.title()),
            force,
        )?;

        copy_research_or_write(
            &format!("lenses/{}/practices.md", lens.as_str()),
            &lens_dir.join("practices.md"),
            format!(
                "# {}\n\nNo source-backed practices were available.\n",
                lens.title()
            ),
            force,
        )?;
        copy_optional_research(
            &format!("lenses/{}/prompt-guidance.md", lens.as_str()),
            &lens_dir.join("prompt-guidance.md"),
            force,
        )?;
        copy_optional_research(
            &format!("lenses/{}/practice-atoms.yaml", lens.as_str()),
            &lens_dir.join("practice-atoms.yaml"),
            force,
        )?;
        copy_optional_research(
            &format!("lenses/{}/coverage-matrix.md", lens.as_str()),
            &lens_dir.join("coverage-matrix.md"),
            force,
        )?;
        copy_optional_research(
            &format!("lenses/{}/research-gaps.md", lens.as_str()),
            &lens_dir.join("research-gaps.md"),
            force,
        )?;
        copy_optional_research(
            &format!("lenses/{}/source-map.yaml", lens.as_str()),
            &lens_dir.join("source-map.yaml"),
            force,
        )?;

        let prompt = lens_prompt(*lens, &lens_dir.join("prompt-guidance.md"));
        write_text_if_absent(&lens_dir.join("prompt.md"), prompt, force)?;
    }

    Ok(())
}

fn seed_optics(root: &Path, force: bool) -> Result<()> {
    let docs_dir = root.join("optics/documentation-knowledge");
    fs::create_dir_all(&docs_dir).with_context(|| format!("create {}", docs_dir.display()))?;
    write_text_if_absent(
        &docs_dir.join("optic.toml"),
        "id = \"documentation-knowledge\"\nname = \"Documentation / Knowledge\"\n",
        force,
    )?;
    copy_research_or_write(
        "optics/documentation-knowledge/practices.md",
        &docs_dir.join("practices.md"),
        "# Documentation / Knowledge\n\nNo source-backed practices were available.\n",
        force,
    )?;
    copy_optional_research(
        "optics/documentation-knowledge/prompt-guidance.md",
        &docs_dir.join("prompt-guidance.md"),
        force,
    )?;
    copy_optional_research(
        "optics/documentation-knowledge/practice-atoms.yaml",
        &docs_dir.join("practice-atoms.yaml"),
        force,
    )?;
    copy_optional_research(
        "optics/documentation-knowledge/source-map.yaml",
        &docs_dir.join("source-map.yaml"),
        force,
    )?;
    write_text_if_absent(
        &docs_dir.join("prompt.md"),
        optic_prompt(
            Optic::DocumentationKnowledge,
            "Use documentation as an operational knowledge system. Do not file writing-style preferences unless they affect onboarding, delivery, support, safety, compliance, or operations.",
        ),
        force,
    )?;

    let nice_dir = root.join("optics/nice-practices");
    fs::create_dir_all(&nice_dir).with_context(|| format!("create {}", nice_dir.display()))?;
    write_text_if_absent(
        &nice_dir.join("optic.toml"),
        "id = \"nice-practices\"\nname = \"Nice Practices\"\n",
        force,
    )?;
    write_text_if_absent(
        &nice_dir.join("practices.md"),
        "# Nice Practices\n\nNo substantive personal Nice Practices are defined for v1 yet. Treat this optic as a placeholder. Do not invent preferences. File findings only when the project-local config or future pack version defines explicit personal practices.\n",
        force,
    )?;
    write_text_if_absent(
        &nice_dir.join("prompt.md"),
        optic_prompt(
            Optic::NicePractices,
            "This v1 pack intentionally contains no substantive Nice Practices. Do not create findings from generic taste or inferred personal preference.",
        ),
        force,
    )?;

    Ok(())
}

fn seed_overlays(root: &Path, force: bool) -> Result<()> {
    let Some(research_root) = research_root() else {
        return Ok(());
    };
    let stacks_dir = research_root.join("stacks");
    if !stacks_dir.exists() {
        return Ok(());
    }

    for entry in
        fs::read_dir(&stacks_dir).with_context(|| format!("read {}", stacks_dir.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", stacks_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if matches!(stem, "language-stack-index" | "favorite-practices") {
            continue;
        }

        let overlay_dir = root.join("overlays").join(stem);
        fs::create_dir_all(&overlay_dir)
            .with_context(|| format!("create {}", overlay_dir.display()))?;
        write_text_if_absent(
            &overlay_dir.join("overlay.toml"),
            format!("id = \"{stem}\"\nname = \"{stem}\"\n"),
            force,
        )?;
        copy_file_if_needed(&path, &overlay_dir.join("practices.md"), force)?;
    }

    Ok(())
}

fn seed_integration(root: &Path, force: bool) -> Result<()> {
    for name in [
        "confidence-model.md",
        "deduplication-rules.md",
        "evidence-model.md",
        "final-editor-checklist.md",
        "final-gaps.md",
        "lens-boundaries.md",
        "local-handbook-integration.md",
        "severity-model.md",
    ] {
        copy_optional_research(
            &format!("integration/{name}"),
            &root.join("integration").join(name),
            force,
        )?;
    }

    Ok(())
}

fn copy_research_or_write(
    relative: &str,
    target: &Path,
    fallback: impl AsRef<str>,
    force: bool,
) -> Result<()> {
    if let Some(source) = research_root().map(|root| root.join(relative)) {
        if source.exists() {
            return copy_file_if_needed(&source, target, force);
        }
    }

    write_text_if_absent(target, fallback, force)
}

fn copy_optional_research(relative: &str, target: &Path, force: bool) -> Result<()> {
    if let Some(source) = research_root().map(|root| root.join(relative)) {
        if source.exists() {
            copy_file_if_needed(&source, target, force)?;
        }
    }

    Ok(())
}

fn copy_file_if_needed(source: &Path, target: &Path, force: bool) -> Result<()> {
    if target.exists() && !force {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(source, target)
        .with_context(|| format!("copy {} to {}", source.display(), target.display()))?;
    Ok(())
}

fn research_root() -> Option<PathBuf> {
    let cwd = PathBuf::from(".local/research");
    if cwd.exists() {
        return Some(cwd);
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local/research");
    manifest.exists().then_some(manifest)
}

fn lens_prompt(lens: Lens, prompt_guidance_path: &Path) -> String {
    let guidance = read_optional(prompt_guidance_path)
        .ok()
        .flatten()
        .unwrap_or_else(|| "No lens-specific prompt guidance was found.".to_owned());

    format!(
        r#"# {title} Lens Prompt

{{{{ base_reviewer_guidance }}}}

## Lens-Specific Guidance

{guidance}

## Lens Practices

```text
{{{{ lens_practices }}}}
```

## Assigned Work

{template}
"#,
        title = lens.title(),
        template = lens_review_fallback()
    )
}

fn optic_prompt(optic: Optic, guidance: &str) -> String {
    format!(
        r#"# {title} Optic Prompt

{{{{ base_reviewer_guidance }}}}

## Optic-Specific Guidance

{guidance}

## Optic Practices

```text
{{{{ optic_practices }}}}
```

## Assigned Work

Domain ID: `{{{{ domain_id }}}}`
Domain name: `{{{{ domain_name }}}}`
Optic ID: `{{{{ optic_id }}}}`

Repository context:

```text
{{{{ repository_context }}}}
```

Domain context:

```text
{{{{ domain_context }}}}
```

Output paths:

```text
{{{{ output_paths }}}}
```

Review this domain through the assigned supplemental optic. File only evidence-backed findings connected to the domain.
"#,
        title = optic.title()
    )
}

fn domain_discovery_prompt() -> &'static str {
    r#"# Domain Discovery

{{ base_reviewer_guidance }}

## Repository Context

```text
{{ repository_context }}
```

## Task

Split this repository into reviewable domains or subdomains. Prefer stable ownership and responsibility boundaries over implementation trivia.

Each domain should include:

- stable `domain_id`;
- human-readable name;
- responsibility;
- key files and directories;
- neighboring domains;
- external dependencies;
- possible risk areas;
- recommended lenses.

## Output Paths

```text
{{ output_paths }}
```

Write `domain-map.md` for humans and `domain-map.yaml` as structured data.
"#
}

fn cross_domain_prompt() -> &'static str {
    r#"# Cross-Domain Review

{{ base_reviewer_guidance }}

## Repository Context

```text
{{ repository_context }}
```

## Domain Map

```text
{{ domain_map }}
```

## Task

Look for issues between domains: cyclic dependencies, implicit ownership, duplicated business logic, conflicting assumptions, incompatible contracts, trust-boundary gaps, shared bottlenecks, and inconsistent operational behavior.

## Output Paths

```text
{{ output_paths }}
```
"#
}

fn base_reviewer_fallback() -> &'static str {
    "# Base Reviewer Guidance\n\nBe evidence-first. Do not file generic advice. Separate severity from confidence.\n"
}

fn lens_review_fallback() -> &'static str {
    r#"# Lens Review Template

Domain ID: `{{ domain_id }}`
Domain name: `{{ domain_name }}`
Lens ID: `{{ lens_id }}`

Repository context:

```text
{{ repository_context }}
```

Domain context:

```text
{{ domain_context }}
```

Output paths:

```text
{{ output_paths }}
```

Produce a markdown report, structured findings, and reviewer notes.
"#
}

fn domain_synthesis_fallback() -> &'static str {
    "# Domain Synthesis\n\nMerge findings for `{{ domain_id }}`. Deduplicate, calibrate severity/confidence, and produce a remediation sequence.\n\n{{ domain_findings }}\n"
}

fn system_synthesis_fallback() -> &'static str {
    "# System Synthesis\n\nBuild a system-level report from domain synthesis reports and cross-domain findings.\n\n{{ synthesis_inputs }}\n"
}

fn previous_runs_fallback() -> &'static str {
    "# Previous Runs Comparison\n\nCompare current findings with previous run findings. Preserve old high-risk findings unless clearly fixed.\n\n{{ previous_run_context }}\n"
}

fn final_editor_fallback() -> &'static str {
    "# Final Editor\n\nVerify that the final report is evidence-backed, deduplicated, calibrated, and actionable.\n\n{{ final_editor_inputs }}\n"
}

fn missing_pack_file(path: &Path) -> String {
    format!(
        "# Missing Pack File\n\nThe expected pack file `{}` was not found.\n",
        path.display()
    )
}
