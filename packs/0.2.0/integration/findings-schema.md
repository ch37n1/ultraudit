# Findings File Schema

Status: batch-2 baseline

Every reviewer step must write a structured findings file. Use `[]` when there are no accepted findings.

The file must be a YAML sequence. Each accepted finding must include these common fields:

- `id`: stable, uppercase or kebab-like identifier unique within the file.
- `title`: concise problem statement.
- `domain`: reviewed domain id or `system`.
- `primary_lens`: lens or optic that owns the root cause.
- `secondary_tags`: YAML sequence, can be empty.
- `severity`: one of `critical`, `high`, `medium`, `low`, `info`.
- `confidence`: one of `high`, `medium`, `low`.
- `status`: `open`, `fixed`, `needs-recheck`, `accepted-risk`, or `hypothesis`.
- `evidence`: YAML sequence of path/line/detail objects when local evidence exists.
- `impact`: concrete affected workflow, user, data, security property, reliability property, or operator outcome.
- `recommendation`: root-cause remediation.
- `uncertainty`: remaining assumptions or missing external evidence.
- `false_positive_risks`: YAML sequence, can be empty.
- `practice_references`: YAML sequence, can be empty.

Do not put prose-only findings in this file. Put narrative detail in the markdown report and keep this file machine-consumable.
