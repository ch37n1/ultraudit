# ultraudit
Tool to audit applications

## Implementation

Ultraudit is a Rust CLI orchestrator for repeatable agentic application audits. The v1 flow can:

- seed a user-level prompt/practice pack from `.local/research`;
- create project-local `.audit/config.toml` and agent runner config;
- collect repository intake and deterministic fallback domains;
- run domain discovery, lens/optic reviews, cross-domain review, domain synthesis, system synthesis, previous-run comparison, and final editorial verification;
- preserve prompts, invocation manifests, command summaries, stdout/stderr, exit metadata, findings files, reports, suggestions, and a prompt-pack snapshot under each run directory.

Typical commands:

```bash
cargo run -- init
cargo run -- run --pack full
cargo run -- run --dry-run --pack production
cargo run -- run --agent codex --lens security --domain auth
```

The default `codex` agent uses a typed Codex CLI invocation. Unknown agents can be configured as shell-template runners under `.audit/agents/`. The `nice-practices` optic exists as a v1 placeholder only; it intentionally contains no substantive personal practices yet.

## Documentation

The repository language is English. Documentation is split by language:

- `docs/ru/` - Russian working drafts. Use this when the initial discussion or design work happens in Russian.
- `docs/en/` - English canonical documentation. Once a Russian draft stabilizes, translate it here and continue future work in English.

Current documents:

- [Vision](docs/en/vision.md)
- [Vision (RU)](docs/ru/vision.md)
- [English docs index](docs/en/README.md)
