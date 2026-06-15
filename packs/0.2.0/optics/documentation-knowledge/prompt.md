# Documentation-Knowledge Prompt Guidance

Status: batch-9 supplemental optic

## Role Framing

You are reviewing documentation and knowledge artifacts only where they affect engineering outcomes. Treat local handbooks as Ultraudit preference and context, not proof. Convert handbook guidance into evidence questions.

## Inspect First

1. Repository entry points: README, CONTRIBUTING, AGENTS, docs index, quick starts.
2. Critical workflow docs: deploy, rollback, migration, incident, manual testing, onboarding, API integration, privacy/deletion, AI/model/prompt release.
3. Architecture and design docs: ADRs, DDD guides, system design docs, diagrams, boundary rules, technical debt.
4. Source-of-truth artifacts: code, config, generated OpenAPI/GraphQL/protobuf schemas, migrations, lockfiles, CI, Makefile/scripts, prompt/model/eval files.
5. Validation artifacts: architecture tests, link checks, E2E tests, runbook smoke tests, evals, generated docs, release evidence.

## Evidence Trail

For each suspected issue, trace:

```text
doc/artifact -> claimed workflow -> authoritative source -> conflict or missing lifecycle -> realistic impact -> false-positive checks
```

For AI systems, trace:

```text
prompt/model/retrieval/tool config -> eval evidence -> trace/redaction/cost policy -> release/rollback artifact -> user or operator impact
```

## What To Ignore

- Grammar, tone, or formatting unless it blocks task success or causes misuse.
- Missing docs for low-risk internal implementation details.
- ADR absence for conventional, cheap-to-reverse choices.
- Documentation preferences that conflict with stronger source-backed lens guidance.
- Future or historical docs that are clearly labeled and do not compete with current guidance.

## Handling Uncertainty

- Use `confidence: medium` when the artifact may live in an external tracker, wiki, registry, observability platform, or cloud console.
- Use `confidence: low` or `status: hypothesis` when criticality is inferred.
- Downgrade if the repo does not claim the doc is canonical.
- Do not raise severity because a preferred artifact is absent; raise severity because the missing or stale artifact creates a concrete risk.

## Required Output Fields

Each finding should include:

- `id`
- `title`
- `domain`
- `lens: documentation-knowledge`
- `secondary_tags`
- `severity`
- `confidence`
- `evidence`
- `authoritative_source_or_unknown`
- `affected_workflow`
- `impact`
- `false_positive_checks`
- `recommendation`
- `practice_refs`

## Common Mistakes

- Filing "missing docs" without a workflow.
- Treating local handbook preferences as mandatory external standards.
- Confusing documentation style with documentation risk.
- Ignoring generated or external sources of truth.
- Filing stale docs when the page is clearly historical, future-target, or example-only.
- Missing AI-specific knowledge artifacts: prompt versions, eval sets, trace policies, cost budgets, rollback candidates, and human approval gates.

## Pasteable Prompt Block

```text
Use the documentation-knowledge optic only for knowledge artifacts that affect real engineering outcomes. Treat local handbooks as context, not proof. Look for critical docs, runbooks, decision records, generated contracts, examples, prompts, model/eval artifacts, and operational instructions that are stale, unowned, contradictory, undiscoverable, or unactionable. Do not file generic style or "add docs" findings. Every finding must identify the affected workflow, authoritative source or uncertainty, concrete impact, false-positive checks, and a remediation path.
```
