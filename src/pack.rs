use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    cli::{Lens, LensPack, Optic},
    toml::SimpleToml,
    util::{ensure_parent, relative_display},
};

const MAX_PACK_FILES: usize = 2_000;
const MAX_PACK_BYTES: u64 = 50 * 1024 * 1024;
const PACK_SCHEMA_VERSION: &str = "2";

#[derive(Debug, Clone)]
pub struct Pack {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackPolicy {
    sets: BTreeMap<String, Vec<Lens>>,
    project_optics: Vec<Optic>,
    cross_system_lenses: Vec<Lens>,
    synthesis_integration: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PackSnapshot {
    pub path: PathBuf,
    pub content_fingerprint: String,
    pub file_count: usize,
    pub byte_count: u64,
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

    pub fn lens_evidence(&self, lens: Lens) -> PathBuf {
        self.root
            .join("lenses")
            .join(lens.as_str())
            .join("evidence.md")
    }

    pub fn lens_false_positives(&self, lens: Lens) -> PathBuf {
        self.root
            .join("lenses")
            .join(lens.as_str())
            .join("false-positives.md")
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

    pub fn optic_evidence(&self, optic: Optic) -> PathBuf {
        self.root
            .join("optics")
            .join(optic.as_str())
            .join("evidence.md")
    }

    pub fn optic_false_positives(&self, optic: Optic) -> PathBuf {
        self.root
            .join("optics")
            .join(optic.as_str())
            .join("false-positives.md")
    }

    pub fn integration(&self, name: &str) -> PathBuf {
        self.root.join("integration").join(format!("{name}.md"))
    }
}

pub fn pack_root(prompt_home: &Path, _name: &str, version: &str) -> PathBuf {
    prompt_home.join("packs").join(version)
}

pub fn legacy_pack_root(prompt_home: &Path, name: &str, version: &str) -> PathBuf {
    prompt_home
        .join("packs")
        .join(name)
        .join("versions")
        .join(version)
}

pub fn load_optional_pack_policy(root: &Path) -> Result<Option<PackPolicy>> {
    let manifest = root.join("pack.toml");
    if !manifest.exists() {
        return Ok(None);
    }

    Ok(Some(read_pack_policy(root, None)?))
}

pub fn read_pack_policy(root: &Path, expected_version: Option<&str>) -> Result<PackPolicy> {
    let manifest = root.join("pack.toml");
    let parsed = SimpleToml::read(&manifest)?;

    let schema_version = parsed.value("", "schema_version").unwrap_or_default();
    if schema_version != PACK_SCHEMA_VERSION {
        bail!(
            "prompt pack manifest {} must set schema_version = \"{}\"",
            manifest.display(),
            PACK_SCHEMA_VERSION
        );
    }

    if let Some(expected_version) = expected_version {
        let actual_version = parsed.value("", "version").unwrap_or_default();
        if actual_version != expected_version {
            bail!(
                "prompt pack manifest {} declares version `{actual_version}`, expected `{expected_version}`",
                manifest.display()
            );
        }
    }

    let mut sets = BTreeMap::new();
    if let Some(values) = parsed.sections.get("sets") {
        for key in values.keys() {
            let lenses = parse_lens_array(&parsed, "sets", key)
                .with_context(|| format!("parse pack set `{key}` in {}", manifest.display()))?;
            sets.insert(key.clone(), lenses);
        }
    }

    let project_optics = parse_optic_array(&parsed, "flows.project_review", "optics")
        .with_context(|| format!("parse project review optics in {}", manifest.display()))?;
    let cross_system_lenses = parse_lens_array(&parsed, "flows.cross_system_review", "lenses")
        .with_context(|| format!("parse cross-system lenses in {}", manifest.display()))?;
    let synthesis_integration = parsed.array("flows.synthesis", "integration");

    Ok(PackPolicy {
        sets,
        project_optics: non_empty_or_default(project_optics, Optic::all_default()),
        cross_system_lenses: non_empty_or_default(
            cross_system_lenses,
            Lens::cross_system_default(),
        ),
        synthesis_integration: if synthesis_integration.is_empty() {
            default_synthesis_integration()
        } else {
            synthesis_integration
        },
    })
}

impl PackPolicy {
    pub fn builtin() -> Self {
        let mut sets = BTreeMap::new();
        for pack in [
            LensPack::Default,
            LensPack::Production,
            LensPack::ContractsAndData,
            LensPack::Product,
            LensPack::Full,
        ] {
            sets.insert(pack.as_str().to_owned(), pack.lenses().to_vec());
        }

        Self {
            sets,
            project_optics: Optic::all_default().to_vec(),
            cross_system_lenses: Lens::cross_system_default().to_vec(),
            synthesis_integration: default_synthesis_integration(),
        }
    }

    pub fn lenses_for_set(&self, name: &str) -> Option<&[Lens]> {
        self.sets.get(name).map(Vec::as_slice)
    }

    pub fn project_optics(&self) -> &[Optic] {
        &self.project_optics
    }

    pub fn cross_system_lenses(&self) -> &[Lens] {
        &self.cross_system_lenses
    }

    pub fn synthesis_integration(&self) -> &[String] {
        &self.synthesis_integration
    }
}

pub fn validate_pack_files(pack: &Pack, policy: &PackPolicy) -> Result<()> {
    for prompt in [
        "base-reviewer",
        "cross-system-review",
        "domain-discovery",
        "domain-lens-review",
        "domain-synthesis",
        "final-editor",
        "previous-runs-comparison",
        "project-optic-review",
        "system-synthesis",
    ] {
        ensure_pack_file(&pack.prompt(prompt))?;
    }

    for lens in Lens::all() {
        ensure_pack_file(&pack.lens_prompt(*lens))?;
        ensure_pack_file(&pack.lens_practices(*lens))?;
        ensure_pack_file(&pack.lens_evidence(*lens))?;
        ensure_pack_file(&pack.lens_false_positives(*lens))?;
    }

    for optic in Optic::all_default() {
        ensure_pack_file(&pack.optic_prompt(*optic))?;
        ensure_pack_file(&pack.optic_practices(*optic))?;
        ensure_pack_file(&pack.optic_evidence(*optic))?;
        ensure_pack_file(&pack.optic_false_positives(*optic))?;
    }

    for integration in policy.synthesis_integration() {
        ensure_pack_file(&pack.integration(integration))?;
    }
    ensure_pack_file(&pack.integration("final-editor-checklist"))?;
    Ok(())
}

pub fn snapshot_pack(pack: &Pack, run_dir: &Path) -> Result<PackSnapshot> {
    let snapshot = run_dir.join("prompt-pack");
    let mut stats = PackCopyStats::default();
    copy_pack_recursive(&pack.root, &pack.root, &snapshot, &mut stats)?;
    Ok(PackSnapshot {
        path: snapshot,
        content_fingerprint: format!("fnv1a64:{:016x}", stats.fingerprint),
        file_count: stats.file_count,
        byte_count: stats.byte_count,
    })
}

pub fn render_template(template: &str, values: &BTreeMap<&str, String>) -> String {
    let mut rendered = template.to_owned();
    for _ in 0..8 {
        let before = rendered.clone();
        for (key, value) in values {
            rendered = rendered.replace(&format!("{{{{ {key} }}}}"), value);
            rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
        }
        if rendered == before {
            break;
        }
    }
    rendered
}

pub fn read_pack_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read required pack file {}", path.display()))
}

fn parse_lens_array(parsed: &SimpleToml, section: &str, key: &str) -> Result<Vec<Lens>> {
    parsed
        .array(section, key)
        .into_iter()
        .map(|value| Lens::from_id(&value).with_context(|| format!("unknown lens `{value}`")))
        .collect()
}

fn parse_optic_array(parsed: &SimpleToml, section: &str, key: &str) -> Result<Vec<Optic>> {
    parsed
        .array(section, key)
        .into_iter()
        .map(|value| Optic::from_id(&value).with_context(|| format!("unknown optic `{value}`")))
        .collect()
}

fn non_empty_or_default<T: Copy>(values: Vec<T>, fallback: &[T]) -> Vec<T> {
    if values.is_empty() {
        fallback.to_vec()
    } else {
        values
    }
}

fn default_synthesis_integration() -> Vec<String> {
    [
        "findings-schema",
        "evidence-model",
        "severity-model",
        "confidence-model",
        "deduplication-rules",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

fn ensure_pack_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("required prompt pack file {} is missing", path.display()))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!("prompt pack file {} must not be a symlink", path.display());
    }
    if !file_type.is_file() {
        bail!("prompt pack path {} must be a regular file", path.display());
    }
    Ok(())
}

#[derive(Default)]
struct PackCopyStats {
    file_count: usize,
    byte_count: u64,
    fingerprint: u64,
}

fn copy_pack_recursive(
    root: &Path,
    source: &Path,
    target: &Path,
    stats: &mut PackCopyStats,
) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("create {}", target.display()))?;

    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", source.display()))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .with_context(|| format!("metadata {}", source_path.display()))?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            bail!(
                "prompt pack snapshot refuses symlink {}",
                source_path.display()
            );
        }

        if file_type.is_dir() {
            copy_pack_recursive(root, &source_path, &target_path, stats)?;
        } else if file_type.is_file() {
            stats.file_count += 1;
            stats.byte_count = stats.byte_count.saturating_add(metadata.len());
            if stats.file_count > MAX_PACK_FILES || stats.byte_count > MAX_PACK_BYTES {
                bail!(
                    "prompt pack at {} exceeds snapshot limits: {} files, {} bytes",
                    root.display(),
                    stats.file_count,
                    stats.byte_count
                );
            }

            let bytes = fs::read(&source_path)
                .with_context(|| format!("read {}", source_path.display()))?;
            update_fingerprint(stats, relative_display(&source_path, root).as_bytes());
            update_fingerprint(stats, &bytes);

            ensure_parent(&target_path)?;
            fs::write(&target_path, bytes).with_context(|| {
                format!(
                    "copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        } else {
            bail!(
                "prompt pack snapshot refuses unsupported file type {}",
                source_path.display()
            );
        }
    }

    Ok(())
}

fn update_fingerprint(stats: &mut PackCopyStats, bytes: &[u8]) {
    if stats.fingerprint == 0 {
        stats.fingerprint = 0xcbf2_9ce4_8422_2325;
    }
    for byte in bytes {
        stats.fingerprint ^= u64::from(*byte);
        stats.fingerprint = stats.fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_template_resolves_placeholders_inside_inserted_values() {
        let values = BTreeMap::from([
            ("lens_prompt", "Domain {{ domain_id }}".to_owned()),
            ("domain_id", "core".to_owned()),
        ]);

        let rendered = render_template("{{ lens_prompt }}", &values);

        assert_eq!(rendered, "Domain core");
    }
}
