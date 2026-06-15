# Deduplication Rules

Status: batch-1 baseline

## Exact Duplicates

Merge findings when they describe the same file path, same code path, same defect, and same impact. Keep the strongest evidence and highest justified confidence.

## Same Root Cause Across Multiple Files

Merge into one finding when several symptoms arise from one root cause.

Example: five endpoints lack tenant filtering because one repository method omits tenant scope. Primary finding should target the repository method and list affected endpoints.

## Same Symptom, Different Root Causes

Keep separate findings when the same symptom has different causes.

Example: one endpoint has missing object authorization; another logs PII due to debug middleware. Both expose data but root causes differ.

## Cross-Lens Overlap

Choose one primary lens by root cause:

- exploit path: `security`;
- structural boundary: `architecture`;
- AI behavior/eval/provenance: `ml-ai`;
- transaction/invariant/storage correctness: `data-integrity`;
- external contract: `api-contracts`.

Add secondary tags rather than duplicating.

## Repeated Across Runs

If a finding appears in previous runs:

- preserve original ID when the root cause remains;
- mark status `open`, `fixed`, `needs-recheck`, or `accepted-risk`;
- note whether severity/confidence changed and why;
- do not silently drop old findings.

## Speculative Duplicates

Merge speculative findings into a hypothesis group when evidence is incomplete. Do not let multiple low-confidence variants inflate perceived risk.

