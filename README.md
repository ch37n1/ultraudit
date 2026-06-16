# ultraudit
Tool to audit applications

## Implementation

Ultraudit is a Rust CLI orchestrator for repeatable agentic application audits. The v1 flow can:

- use a versioned prompt/practice pack installed from `packs/0.2.0`;
- create project-local `.audit/config.toml` and agent runner config;
- collect repository intake and deterministic fallback domains;
- run domain discovery, domain lens reviews, project optic reviews, cross-system review, domain synthesis, system synthesis, previous-run comparison, and final editorial verification;
- preserve prompts, invocation manifests, command summaries, stdout/stderr, exit metadata, findings files, reports, suggestions, and a prompt-pack snapshot under each run directory.

Typical commands:

```bash
make install
uat init
uat run --plan
uat run
uat run --dry-run --pack production
ULTRAUDIT_PATH=./for-test uat run --dry-run
uat run --agent codex --lens security --domain auth
```

By default, `uat run` selects every built-in lens. Smaller named packs remain available with `--pack production`, `--pack contracts-and-data`, and `--pack product`, and specific lenses can be selected with repeated `--lens` flags.

The default `codex` agent uses a typed Codex CLI invocation. Set `model = "..."` in `.audit/agents/codex.toml` to pass `--model` to Codex CLI. Unknown agents can be configured as shell-template runners under `.audit/agents/`. The `nice-practices` optic exists as a v1 placeholder only; it intentionally contains no substantive personal practices yet.

`make install` builds the release binary, installs it to `~/.local/bin/uat` by default, and copies `packs/0.2.0` to `~/.ultraudit/packs/0.2.0`. `uat init` only writes project-local `.audit/` configuration and does not generate or copy prompt packs. `ULTRAUDIT_PATH` overrides the default prompt/practice home (`~/.ultraudit`). `run --plan` only prints the resolved plan. `run --dry-run` writes normal run artifacts while faking every agent call, which is intended for automated end-to-end tests.

## Documentation

The repository language is English. Documentation is split by language:

- `docs/ru/` - Russian working drafts. Use this when the initial discussion or design work happens in Russian.
- `docs/en/` - English canonical documentation. Once a Russian draft stabilizes, translate it here and continue future work in English.

Current documents:

- [Vision](docs/en/vision.md)
- [Vision (RU)](docs/ru/vision.md)
- [English docs index](docs/en/README.md)
