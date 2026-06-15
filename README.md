# ultraudit
Tool to audit applications

## Implementation

Ultraudit is now scaffolded as a Rust CLI crate. The first executable surface is intentionally narrow:

```bash
cargo run -- run --dry-run --pack full
cargo run -- --format json run --dry-run --pack production
cargo run -- completions zsh
```

CLI parsing, flags, help text, color-aware output, JSON output, shell completions, and integration-test scaffolding are in place. The actual audit orchestrator is the next implementation step.

## Documentation

The repository language is English. Documentation is split by language:

- `docs/ru/` - Russian working drafts. Use this when the initial discussion or design work happens in Russian.
- `docs/en/` - English canonical documentation. Once a Russian draft stabilizes, translate it here and continue future work in English.

Current documents:

- [Vision](docs/en/vision.md)
- [Vision (RU)](docs/ru/vision.md)
- [English docs index](docs/en/README.md)
