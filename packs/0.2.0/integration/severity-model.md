# Severity Model

Status: batch-1 baseline

Severity measures impact and likelihood, not confidence. A finding can be high severity and low confidence; synthesis should not present it as accepted high severity until evidence improves.

## Critical

Use when a reachable issue can cause one or more of:

- cross-tenant or broad sensitive data exposure;
- account takeover or privileged access at scale;
- remote code execution or production credential compromise;
- destructive data loss or corruption;
- total or systemic outage;
- unsafe AI/agent action with irreversible, privileged, financial, legal, medical, or production impact;
- regulated or safety-critical harm.

Requires concrete reachability or a clearly plausible path.

## High

Use when the issue can materially affect important users, data, business workflows, security, privacy, availability, or AI behavior, but blast radius or exploitability is more constrained than critical.

Examples:

- object-level auth bypass for sensitive records;
- token lifecycle flaw exposing important account actions;
- architecture boundary causing core domain rule divergence;
- AI output can mislead users in important but recoverable decisions;
- RAG exposes restricted internal data to a limited audience.

## Medium

Use when impact is material but constrained, uncertain, recoverable, or requires additional preconditions.

Examples:

- fragile authorization with unclear exploitability;
- missing AI eval coverage for non-critical workflows;
- architectural coupling likely to slow or break future changes;
- logging potentially sensitive but low-volume data with restricted access.

## Low

Use for localized, low-blast-radius, defense-in-depth, hardening, or maintainability risks with limited user/business impact.

## Info

Use for improvement opportunities, documentation gaps, or observations that do not currently support a risk finding.

## Severity Inputs

Calibrate severity using:

- affected asset sensitivity;
- number and type of users affected;
- exploitability or trigger likelihood;
- blast radius;
- reversibility;
- detectability;
- business impact;
- legal/regulatory/safety impact;
- operational recovery cost;
- evidence strength only when it affects likelihood, not as a substitute for confidence.

