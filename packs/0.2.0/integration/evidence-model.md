# Evidence Model

Status: batch-1 baseline

Ultraudit findings must be evidence-first. Generic advice should be rejected or downgraded.

## Minimum Evidence For Accepted Findings

Each accepted finding should include:

- stable finding ID;
- title;
- domain;
- primary lens and secondary tags;
- severity and confidence;
- concrete evidence with file paths and line ranges when available;
- execution path, data flow, dependency path, or architecture boundary;
- actor or trigger;
- affected asset, user, workflow, data, or system property;
- impact;
- recommendation;
- uncertainty and false-positive risks;
- practice references.

## Strong Evidence Examples

- source-to-sink trace for injection;
- actor-to-resource trace for authorization;
- command-to-invariant-to-transaction trace for data/DDD issues;
- prompt/retriever/tool/action trace for AI systems;
- config default-to-production path for unsafe defaults;
- deployment path showing hidden runtime dependency.

## Reject Or Downgrade

Reject or downgrade:

- generic advice with no code path;
- unsupported high-severity claims;
- style preferences without maintainability or defect impact;
- findings with no domain connection;
- "missing tests" without critical behavior or regression risk;
- "AI may hallucinate" without user reliance and missing controls;
- "architecture is unclear" without concrete change, runtime, data, or ownership risk.

## Source References

Practice references support why a pattern matters. They do not replace local evidence. A source-backed practice plus no local evidence is not a finding.

