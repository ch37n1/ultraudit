# Observability Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The observability lens reviews whether production behavior can be detected, understood, debugged, and audited. It covers logs, metrics, traces, events, audit trails, correlation/context propagation, alertability, error context, telemetry quality, redaction, cardinality/cost, and AI/RAG/agent telemetry.

It should not file generic "add logs" findings. Every observability finding should identify the operational question that cannot be answered and the user, incident, security, data, or AI risk that follows.

## Subtopic Taxonomy

- Signals: metrics, logs, traces, events, profiles, baggage.
- User-impact monitoring: SLIs, success rate, latency, freshness, task completion.
- Alerting: actionable paging, symptom alerts, ownership, noise control.
- Distributed tracing: trace context, span boundaries, async propagation, service maps.
- Structured logging: event names, actor/resource/action/outcome, correlation IDs.
- Audit trails: privileged actions, data exports/deletes, approvals, AI tool calls.
- Error context: cause preservation, operation/resource/dependency metadata.
- Telemetry safety: redaction, minimization, retention, access control.
- Cardinality and cost: labels, attributes, sampling, volume control.
- AI observability: model/prompt/index/tool lineage, evals, safety, cost, latency, retrieval, approval.

## High-Value Review Questions

- What user-impacting failure would operators miss?
- Can one request be traced across services, queues, jobs, and AI/tool calls?
- Would logs answer who did what to which resource with what outcome?
- Are alerts actionable and owned?
- Can high-impact actions be audited after the fact?
- Is sensitive data excluded before telemetry export?
- Could a metric label or trace attribute explode cardinality?
- Can a bad AI answer or agent action be reconstructed safely?

## Concrete Signals

- Alerts only on CPU/memory/restarts for user-facing workflow.
- Queue/job boundary loses trace or request ID.
- Logs contain stack trace without domain context.
- Audit events omit actor, target, outcome, or correlation ID.
- Request bodies, tokens, prompts, or retrieved documents are logged.
- Metrics label includes raw path, user ID, email, prompt, or exception string.
- AI telemetry records latency but not model/prompt/index/tool version.

## Anti-Patterns

- "We have logs" as proof of debuggability.
- Cause-only alerts that page without user impact.
- Every service generates a different correlation ID.
- Debug logs as audit logs.
- Full payload logging as observability.
- High-cardinality labels for unique user/request data.
- Provider billing dashboards as AI observability.

## Evidence Requirements

Observability findings need:

- operational question that cannot be answered;
- workflow, incident, or action where this matters;
- telemetry source/config/code path;
- missing field/signal/propagation/alert/redaction;
- blast radius and ownership if known;
- false-positive checks for external APM, service mesh, collectors, dashboards, or platform audit logs.

## Severity Guidance

- `critical`: blind spot prevents detecting or investigating critical outage, security incident, data corruption, privacy exposure, or unsafe AI action.
- `high`: core workflow failure or high-impact action cannot be detected, traced, or reconstructed.
- `medium`: important diagnosis or alerting gap with bounded blast radius.
- `low`: low-risk telemetry quality issue.
- `info`: observability improvement without clear operational risk.

## Confidence Guidance

- `high`: code/config directly shows missing signal, lost context, bad alert, sensitive telemetry, or audit gap.
- `medium`: external observability platform, collector, service mesh, or audit system may provide coverage.
- `low`: operational context or production telemetry unavailable.

## False-Positive Guidance

- Auto-instrumentation can supply traces/log enrichment outside code.
- Alert and dashboard definitions may live in another repository.
- Some systems are intentionally low-risk and do not page humans.
- Sensitive context may be intentionally excluded.
- Platform audit logs may capture actions not visible in app code.

## Remediation Patterns

- Define user-centered SLIs and alert on actionable symptoms.
- Propagate trace/request/job context across all boundaries.
- Use structured logs with safe domain context.
- Emit audit events for high-impact actions.
- Preserve error cause and operation context.
- Redact/minimize telemetry before export.
- Bound metric cardinality and telemetry volume.
- Add AI/RAG/agent telemetry for quality, safety, cost, latency, lineage, and approvals.

## Good Finding Example

Title: Queue worker loses request correlation for payment retries

Evidence summary: API requests enqueue `PaymentRetryJob` with `payment_id` only. The worker logs provider errors with a local job ID but no original request ID, actor, tenant, or trace context. When a retry duplicates a charge, operators cannot connect the provider error to the initiating request or user.

Severity: high for payment workflows.

Confidence: high if no external queue instrumentation adds this context.

## Weak Or Unacceptable Finding Example

"Add more logging."

Reject this. It lacks the operational question, workflow, missing fields, and risk.

## Source Summary

The first-pass observability lens is grounded in OpenTelemetry signals, context propagation, and semantic conventions; W3C Trace Context; Google SRE monitoring, alerting, and troubleshooting; Prometheus instrumentation and alerting; OWASP/NIST logging guidance; and Google AI/ML evaluation and monitoring sources.

