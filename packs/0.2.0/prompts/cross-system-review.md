# Cross-System Review

{{ base_reviewer_guidance }}

## Integration Rules

{{ integration_context }}

## Cross-System Lens Policy

Default cross-system lenses:

```text
{{ cross_system_lenses }}
```

Use these as risk categories only. Do not run a local code-quality or testing review in this pass.

## Repository Context

```text
{{ repository_context }}
```

## Domain Map

```text
{{ domain_map }}
```

## Task

Look for issues between domains and across the system: cyclic dependencies, implicit ownership, duplicated business rules, conflicting assumptions, incompatible contracts, trust-boundary gaps, shared bottlenecks, and inconsistent operational behavior.

## Output Paths

```text
{{ output_paths }}
```
