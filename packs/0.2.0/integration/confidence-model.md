# Confidence Model

Status: batch-1 baseline

Confidence measures how well the evidence supports the finding. It is separate from severity.

## High Confidence

Use when:

- code/config/docs directly show the issue;
- execution or data flow is traced;
- false-positive checks are addressed;
- the impacted asset and scenario are concrete;
- no major external assumption is required.

## Medium Confidence

Use when evidence is strong but depends on one or more unverified factors:

- middleware, gateway, provider, framework, or database policy behavior;
- deployment config outside the repository;
- model registry, eval platform, monitoring, or data governance outside the repository;
- business rule intent that is strongly implied but not explicit.

## Low Confidence

Use when:

- pattern is suspicious but execution path is incomplete;
- intended ownership or policy is unclear;
- the issue depends on runtime data not available;
- source artifacts are missing and cannot be verified locally.

## Hypothesis

Use for plausible risks that need specific verification. Hypotheses should not appear as accepted findings unless marked clearly and separated from confirmed issues.

## Needs Verification

Use when the next check is clear, such as "confirm API gateway enforces tenant policy" or "inspect model registry for eval gate."

## Likely False Positive

Use when evidence initially suggests a problem but a stronger control probably exists. Preserve only if it teaches synthesis or future reviewers what was checked.

