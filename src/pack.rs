use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    cli::{Lens, Optic},
    util::copy_dir_recursive,
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
    fs::read_to_string(path).with_context(|| format!("read required pack file {}", path.display()))
}
