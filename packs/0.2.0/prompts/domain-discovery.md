# Domain Discovery

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
