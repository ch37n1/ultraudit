# Documentation-Knowledge Practices

Status: batch-9 supplemental optic
Freshness date: 2026-06-15

## Scope

This optic reviews documentation and knowledge artifacts when they affect engineering outcomes. It owns stale or conflicting docs, missing source-of-truth links, unowned critical runbooks, lost decision rationale, unactionable metrics, broken commands/examples, and weak AI/agent artifact lineage.

It does not own grammar, prose style, formatting, or generic requests to "add docs" unless those issues block task success, safety, correctness, maintainability, accessibility, performance, security, privacy, or operations.

## Subtopic Taxonomy

- documentation lifecycle: owner, status, last meaningful review, review trigger, archive/delete path;
- source of truth: authoritative artifact, duplicate copies, generated contracts, stale examples;
- docs-as-code: version control, PR review, docs impact, link/example checks;
- operational runbooks: prerequisites, steps, expected outputs, rollback/cleanup, validation;
- decision records: rationale, alternatives, consequences, known costs, review triggers;
- design artifacts: requirements, assumptions, workload model, quality attributes, rollout, operational readiness;
- observability and metrics: signal owner, baseline, threshold, decision, guardrail;
- AI/ML/LLM artifacts: dataset/model/prompt/eval lineage, traces, cost, safety, rollback, intended use;
- discoverability: navigation, entry points, broken links, orphan critical pages;
- archival: superseded docs, historical targets, stale knowledge removal.

## High-Value Review Questions

- Which docs are required to deploy, rollback, recover, onboard, integrate, operate, release AI behavior, or handle regulated/sensitive data?
- For those docs, can a reviewer identify owner, status, last meaningful review, next review trigger, and archive/delete rule?
- Which source is authoritative for volatile facts: code, config, schema, generated OpenAPI, migration, ADR, runbook, model registry, prompt management, observability platform, or external system?
- Can examples, commands, links, and env vars be mechanically checked?
- Does the repository's change flow include docs impact for user-visible behavior, API contracts, runbooks, prompts, models, evals, deployment, and process changes?
- Do architecture and system-design docs connect decisions to requirements, assumptions, quality attributes, tradeoffs, and production signals?
- Do operational artifacts include evidence that someone can act without the original author?
- For AI/agentic behavior, can production behavior be traced to prompt/model/retrieval/tool versions, eval results, trace policy, cost guardrails, and rollback path?

## Concrete Signals

- README, quick start, runbook, or API docs contain missing files, stale endpoints, renamed commands, or non-portable absolute paths.
- A document is cited as canonical but has no lifecycle metadata or source-of-truth link.
- Multiple docs describe current architecture with conflicting status.
- A critical runbook has commands but no expected outputs, cleanup, rollback, pass criteria, or validation evidence.
- Contribution flow requires tests but omits docs impact for public/API/operational/model behavior.
- Architecture docs claim scalable, reliable, secure, or maintainable without measurable scenario, assumption, owner, or review trigger.
- Dashboard, metric, alert, scorecard, or eval result has no owner or decision path.
- Prompt, model, retrieval, or agent tool changes can reach production without eval, trace, cost, safety, or rollback evidence.

## Anti-Patterns

- Treating page count, page views, or update timestamp as quality evidence.
- Copying volatile facts into many docs for convenience.
- Keeping future target, current design, and historical notes in the same navigation path without status.
- Letting architecture decisions live only in chat, commit messages, or oral history.
- Adding dashboards or scorecards without a response plan.
- Treating model registry, prompt management, or tracing as sufficient without release gates and rollback semantics.

## Evidence Requirements

An accepted finding needs:

- path and line evidence for the problematic doc/artifact;
- the authoritative source or evidence that authority is unclear;
- concrete workflow affected;
- why the workflow matters;
- false-positive checks for external docs, issue trackers, observability tools, registries, CODEOWNERS, PR templates, generated artifacts, or platform controls;
- severity and confidence separated;
- practice atom reference.

## Severity Guidance

- `critical`: documentation or artifact drift can cause irreversible data loss, unauthorized action, unsafe model/agent behavior, unrecoverable outage, or regulated-data mishandling.
- `high`: a critical deploy, rollback, recovery, migration, security/privacy, public API, or ML/AI release workflow depends on stale, missing, or contradictory knowledge.
- `medium`: a material engineering workflow is likely to fail, slow down, or rely on expert memory.
- `low`: local broken link, stale command, unclear owner, or status gap with limited blast radius.
- `info`: style, organization, or consistency improvement with no direct workflow risk.

## Confidence Guidance

- `high`: both the local defect and affected workflow are directly evidenced in the repository.
- `medium`: the defect is visible, but ownership, validation, or authoritative source may live outside the repo.
- `low`: the finding depends on inferred criticality or expected process.
- `hypothesis`: use when the artifact looks missing but the repo does not claim to store it.

## False-Positive Guidance

Before filing, check:

- CODEOWNERS, PR templates, contribution docs, release docs, CI jobs, generated artifacts, and test fixtures;
- external systems that may own the artifact: wiki, issue tracker, model registry, prompt platform, observability tool, incident tool, cloud console;
- whether the doc is explicitly future, historical, example-only, or low risk;
- whether a simple app intentionally avoids heavyweight architecture or documentation process.

## Remediation Patterns

- Add lightweight owner/status/review blocks only to critical artifacts.
- Link to source-of-truth artifacts instead of copying volatile facts.
- Generate API examples, config references, and client types where feasible.
- Add link/command/example checks for critical documentation.
- Add ADRs or decision notes for hard-to-reverse choices.
- Add smoke/drill validation for critical runbooks.
- Attach eval, trace, cost, redaction, and rollback evidence to AI releases.
- Archive or mark superseded docs so they do not compete with current guidance.

## Good Finding Examples

Good:

```yaml
id: documentation-knowledge-readme-stale-script-path
title: README points to a non-portable script path outside the repository
lens: documentation-knowledge
severity: low
confidence: high
evidence:
  - file: .local/project-example/README.md
    note: "PDF export instructions reference an absolute path under a different project location."
impact: "A new contributor following README setup may fail the documented doc-export flow."
recommendation: "Use a repo-relative script path and add a link check or quick-start smoke check if this workflow is important."
practice_refs:
  - documentation-knowledge-link-and-command-validity
```

Good:

```yaml
id: documentation-knowledge-critical-runbook-no-pass-criteria
title: Production rollback runbook has no validation or cleanup criteria
lens: documentation-knowledge
severity: high
confidence: medium
evidence:
  - file: docs/runbooks/rollback.md
    note: "Runbook describes rollback commands but no expected output, data/schema caveats, or post-rollback checks."
impact: "Operators may believe rollback succeeded while database or queue state remains incompatible."
recommendation: "Add prerequisites, expected output, validation checks, data/schema caveats, cleanup, and owner."
practice_refs:
  - documentation-knowledge-runbook-actionable-tested
```

## Weak Or Unacceptable Findings

Weak:

- "The project should add more docs."
- "The README is too short."
- "There is no ADR folder."
- "Page views are not tracked."
- "The docs do not follow Google style."

These lack workflow risk, source-of-truth analysis, or materiality.

## Source Summary

Local handbooks supply Ultraudit-specific conventions. External documentation sources support reader-centered documentation types, docs-as-code workflows, single-source-of-truth behavior, and clear developer documentation. Engineering impact must still cite the existing lens packs for architecture, operations, reliability, data integrity, observability, privacy, security, dependency supply chain, API contracts, or ML/AI.

## Project Validation Summary

`.local/project-example/` shows the optic is realistic:

- boundary docs and architecture tests are positive examples of documentation tied to executable feedback;
- Hermes manual smoke testing is a positive example of a runbook with prerequisites, concrete steps, pass criteria, and red-team notes;
- LLM evals and Langfuse docs are positive examples of AI artifact and tracing guidance;
- missing lifecycle metadata across Markdown docs and an absolute stale README path demonstrate likely low/medium documentation-knowledge findings with false-positive checks.
