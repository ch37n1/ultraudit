# Prompt/Practice Pack v0.2 Plan

This document defines a separate implementation pass for reworking prompts and practices. The goal is to stop generating the pack from Rust code and raw `.local/research`, make the pack a normal versioned Git artifact, and compile a complete self-contained prompt for each agent task from the selected lens or optic.

## Current Problem

Today `uat init` mixes several responsibilities:

- creates the project-local `.audit/` directory;
- creates the user-level `~/.ultraudit` directory;
- generates the default pack from Rust code;
- sometimes copies material from `.local/research`.

As a result, the prompt/practice pack is not a reliable Git source of truth. It is a mix of fallback strings in `src/pack.rs`, local research files, and an installed copy in the user's home directory.

Key current areas:

- `src/pack.rs` - `seed_default_pack`, `pack_root`, prompt/practice pack generation;
- `src/orchestrator.rs` - `run_lens_review`, `run_optic_review`, `run_cross_domain_review`, synthesis passes;
- `src/cli.rs` - has `run` and `init`; do not add `ultraudit install`, because the CLI should not install itself.

## Target Model

Research remains source material and is not consumed by agents directly. Agents should consume curated prompt/practice assets prepared from research ahead of time.

The pack source of truth should live in the repository:

```text
packs/
  0.2.0/
    pack.toml
    prompts/
      base-reviewer.md
      domain-discovery.md
      domain-lens-review.md
      project-optic-review.md
      cross-system-review.md
      domain-synthesis.md
      system-synthesis.md
      previous-runs-comparison.md
      final-editor.md
    lenses/
      architecture/
        prompt.md
        practices.md
        evidence.md
        false-positives.md
      code-quality/
      security/
      correctness/
      testing/
      reliability/
      performance/
      observability/
      operations/
      api-contracts/
      data-integrity/
      privacy-compliance/
      dependency-supply-chain/
      ux-product/
      ml-ai/
    optics/
      documentation-knowledge/
        prompt.md
        practices.md
        evidence.md
        false-positives.md
      nice-practices/
    integration/
      evidence-model.md
      severity-model.md
      confidence-model.md
      deduplication-rules.md
      final-editor-checklist.md
```

Installed user-level copy:

```text
~/.ultraudit/
  config.toml
  packs/
    0.2.0/
      ...
```

The extra `packs/ultraudit-default/versions/0.1.0` nesting should be removed for the new version. While there is only one primary pack, `ultraudit-default` does not add useful information. The old layout can remain only as backward compatibility.

## Prompt Assembly Rule

Each agent task receives a complete self-contained prompt for its task, lens or optic, and domain.

Example task:

```text
review domain users through code-quality
```

Receives:

```text
base reviewer contract
+ full code-quality prompt
+ full code-quality practices
+ code-quality evidence requirements
+ code-quality false-positive checks
+ severity/confidence model
+ task description
+ domain context
+ project/domain map
+ output schema and paths
```

This is not a global knowledge base that the agent must explore by itself. It is also not a short summary. It is a full task prompt deterministically assembled from versioned pack blocks.

Raw research artifacts, source maps, coverage matrices, and research gaps may live alongside the pack for maintenance, but they should not be automatically included in runtime prompts unless they have been turned into agent-facing guidance.

## Review Flow Separation

The review flow types should be explicit.

### 1. Domain Lens Review

Primary pass: one domain or subdomain plus one lens.

Example:

```text
domain: users
lens: code-quality
```

The agent receives all practices and review rules for `code-quality`, the project map, domain description, and the files to inspect first.

Candidate domain-level lenses:

```text
architecture
code-quality
security
correctness
testing
reliability
performance
observability
operations
api-contracts
data-integrity
privacy-compliance
dependency-supply-chain
ux-product
ml-ai
```

### 2. Project Optic Review

Some optics are not naturally domain-scoped. For example, `documentation-knowledge` should be reviewed at the project or system level, not separately for every domain.

Candidates:

```text
documentation-knowledge
nice-practices
```

These optics need a separate prompt template such as `project-optic-review.md`.

### 3. Cross-System Review

After domain-level reviews, a second-order system-wide pass is needed. This pass does not need every lens.

Default cross-system review lenses:

```text
architecture
security
reliability
operations
api-contracts
data-integrity
privacy-compliance
ml-ai
```

Exclude by default:

```text
code-quality
testing
documentation-knowledge
nice-practices
```

Reason: code quality and testing usually require concrete local file inspection and local practices. Documentation is a project-level optic, not a cross-system risk lens.

### 4. Synthesis And Final Editor

Synthesis passes should not receive all lens practices. They need:

```text
evidence model
severity/confidence model
deduplication rules
report contract
previous-run comparison rules
final editor checklist
```

Their job is to merge, deduplicate, calibrate, and edit findings, not to search for problems through the lenses again.

## Pack Manifest

`packs/0.2.0/pack.toml` should express flow policy.

Example:

```toml
schema_version = "2"
version = "0.2.0"

[sets]
default = ["architecture", "code-quality", "security", "correctness", "testing"]
production = ["reliability", "performance", "observability", "operations"]
contracts-and-data = ["api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain"]
product = ["ux-product", "ml-ai"]
full = ["architecture", "code-quality", "security", "correctness", "testing", "reliability", "performance", "observability", "operations", "api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain", "ux-product", "ml-ai"]

[flows.domain_review]
lenses = ["architecture", "code-quality", "security", "correctness", "testing"]

[flows.project_review]
optics = ["documentation-knowledge", "nice-practices"]

[flows.cross_system_review]
lenses = ["architecture", "security", "reliability", "operations", "api-contracts", "data-integrity", "privacy-compliance", "ml-ai"]

[flows.synthesis]
integration = ["evidence-model", "severity-model", "confidence-model", "deduplication-rules"]
```

Later, CLI flags such as `--flow domain`, `--flow project`, or `--flow cross-system` can be added. For the first pass, the manifest can be used as internal policy.

## Installation Model

Installation should happen outside the CLI, through Make:

```bash
git clone <repo-url>
cd ultraudit
make install
```

`ultraudit` should not have an `install` command: the console utility should not install itself. It should only use an already installed prompt/practice pack.

`make install` should:

- build the release binary;
- install the binary into a user-level bin directory;
- create `~/.ultraudit`;
- copy Git-tracked packs from the repository;
- check that `codex` is available in `PATH`, and print a clear error or warning if it is unavailable;
- not depend on `.local/research`.

Pack copy:

```text
repo/packs/0.2.0
```

to:

```text
~/.ultraudit/packs/0.2.0
```

Choose the binary install path explicitly. Preferred path:

```text
~/.local/bin/uat
```

If `~/.local/bin` is not in `PATH`, `make install` should print a post-install instruction.

`init` should only create project-local configuration:

```text
.audit/config.toml
.audit/agents/codex.toml
.audit/agents/custom-shell.toml
```

Example `.audit/config.toml` for the new version:

```toml
[prompt_pack]
version = "0.2.0"
source = "~/.ultraudit/packs/0.2.0"

[run]
agent = "codex"
output_dir = ".audit-runs"
disabled_optics = []
```

Remove `name = "ultraudit-default"` from the new configuration, or keep it only as a deprecated compatibility field.

## Implementation Plan

1. Create `packs/0.2.0` in the repository.
2. Move curated prompt/practice assets there from `.local/research`, but not raw research.
3. Add a `Makefile` with an `install` target.
4. In `make install`:
   - run `cargo build --release`;
   - install `target/release/uat` to `~/.local/bin/uat` or another chosen user bin;
   - create `~/.ultraudit/packs`;
   - copy `packs/0.2.0` to `~/.ultraudit/packs/0.2.0`;
   - check `command -v codex`;
   - print a post-install summary.
5. Rewrite `src/pack.rs`:
   - remove primary pack generation from Rust;
   - change `pack_root` to `~/.ultraudit/packs/{version}`;
   - keep the old layout only as fallback.
6. Update `init_project`:
   - do not seed the pack;
   - write `.audit/config.toml` for the new structure.
7. Update `resolve_pack`:
   - read explicit `source` first;
   - then look for `~/.ultraudit/packs/{version}`;
   - if the pack is missing, return a clear error: run `make install` from the Ultraudit repository.
8. Update runtime prompt assembly:
   - `run_lens_review` assembles the full prompt from lens files;
   - use `run_optic_review` for project-level optics, or split it into a new flow;
   - `cross-domain`, `system-synthesis`, and `final-editor` use only integration prompts.
9. Update selection policy:
   - domain-level lenses separately;
   - project-level optics separately;
   - cross-system lenses separately.
10. Update snapshot behavior:
   - every run copies the resolved installed pack to `run_dir/prompt-pack`;
   - compiled prompts remain in `raw/*/prompt.md`.
11. Update README:
   - `make install`
   - `uat init`
   - `uat run --pack default`

## Tests

Add or update tests:

- `make install` builds the release binary;
- `make install` installs the binary into the user-level bin directory;
- `make install` copies `packs/0.2.0` to selected `ULTRAUDIT_PATH` or `~/.ultraudit`;
- `make install` checks that `codex` is available;
- installed files match checked-in pack assets;
- `uat init` does not require `.local/research`;
- `uat run --plan` or `run --dry-run` resolves `~/.ultraudit/packs/0.2.0`;
- compiled prompt for a domain-lens task contains the full guide for the selected lens;
- `documentation-knowledge` does not run per-domain by default;
- cross-system review does not run `code-quality` by default;
- fresh checkout without `.local/research` passes install/init/plan.

## Acceptance Criteria

A fresh checkout without `.local/research` must support:

```bash
make install
uat init
uat run --plan
```

After `make install`, the `uat` command must be available from the shell, and `~/.ultraudit/packs/0.2.0` must contain a copy of the Git-tracked pack.

The runtime prompt for a domain-lens task must be self-contained and include the full curated guide for the selected lens or optic, task context, domain/project map, and output contract.

The installed pack must be a copy of Git-tracked `packs/0.2.0`, not generated from Rust and not copied from raw research.
