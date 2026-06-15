# Reliability Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The reliability lens reviews whether the application keeps serving acceptable user outcomes when dependencies are slow or failing, traffic spikes, deploys happen, retries occur, workers crash, queues back up, health checks route traffic, or recovery paths are exercised.

It owns timeouts, deadlines, retries, backoff, jitter, idempotency for recovery, overload handling, backpressure, load shedding, circuit breaking, fallback/failover, health/readiness behavior, graceful degradation, SLO alignment, recovery testing, and repeated incident prevention.

## Subtopic Taxonomy

- User-visible reliability targets: SLIs, SLOs, error budgets, freshness, latency, success rate.
- Dependency failure: timeouts, deadlines, fail-fast behavior, dependency health.
- Retry safety: retry budgets, idempotency keys, duplicate side effects, retry layers.
- Overload: bounded queues, backpressure, admission control, load shedding, quotas.
- Cascading failure: resource exhaustion, load redistribution, retry storms, shared dependency collapse.
- Failure containment: circuit breakers, bulkheads, per-shard/provider/tenant scoping.
- Degradation and fallback: stale data, reduced accuracy, alternate providers, feature disablement.
- Health and readiness: liveness, readiness, startup, warmup, orchestration routing.
- Background jobs: replay, deduplication, poison messages, dead-letter behavior.
- Recovery and launch readiness: rollback, failover, replay, restore, fault injection, postmortems.
- Archetype variations: mobile offline/sync, CLI exit/retry semantics, desktop local state recovery, AI provider/model fallback and cost budgets.

## High-Value Review Questions

- Which user workflow must remain available or degrade cleanly during dependency failure?
- Are all remote calls bounded by timeouts or deadlines?
- Can a mutation be retried safely after timeout or unknown result?
- Are retries bounded, jittered, and applied at only one sensible layer?
- What resource exhausts first under overload?
- Are queues, concurrency, model calls, and background work bounded?
- Can health checks accidentally restart or route traffic to bad instances?
- Are fallback paths exercised, observable, and safe?
- Are recovery paths tested with assertions, not just described?

## Concrete Signals

- Remote client call without explicit timeout.
- Retry loop around mutation with no idempotency key.
- Nested retries in SDK, wrapper, job runner, and workflow.
- Unbounded queue or task spawn in request path.
- Cache failure falls back to database at full traffic.
- Same health endpoint used for readiness and liveness.
- Job can crash after side effect but before ack with no dedup.
- No alert for retry rate, queue age, load shedding, or degraded mode.

## Anti-Patterns

- "Retry until it works."
- Autoscaling as the only overload plan.
- Fallback path that is rarely used and more expensive than the primary path.
- Health endpoint that always returns 200.
- Exactly-once assumptions for queues and distributed workflows.
- SLOs based only on process uptime or infrastructure metrics.
- Recovery runbook not exercised in tests or production drills.

## Evidence Requirements

Reliability findings should include:

- affected workflow and user-visible reliability expectation;
- code/config path for call, retry, queue, probe, fallback, or recovery behavior;
- dependency or resource involved;
- failure scenario;
- blast radius;
- false-positive checks for gateway, SDK, platform, service mesh, queue broker, or external runbooks;
- recommendation tied to recovery or degradation.

## Severity Guidance

- `critical`: plausible failure causes systemic outage, unrecoverable data loss, duplicate destructive action, or unsafe high-impact AI/automation behavior.
- `high`: core workflow can hang, collapse, duplicate side effects, or fail recovery under common dependency/load failures.
- `medium`: important reliability control is weak but blast radius is bounded.
- `low`: hardening or observability gap for low-impact path.
- `info`: reliability improvement without demonstrated failure impact.

## Confidence Guidance

- `high`: code/config directly shows missing timeout, unsafe retry, unbounded queue, bad probe, or untested recovery path.
- `medium`: platform/SDK/external configuration may provide the missing control.
- `low`: plausible reliability risk with incomplete runtime topology or traffic assumptions.

## False-Positive Guidance

- SDKs may have default retries, timeouts, and retry budgets.
- Gateways, service meshes, queue brokers, and orchestrators may enforce controls outside the repo.
- Small CLIs and local tools may not need SLOs or load shedding.
- Some manual recovery is acceptable for low-frequency, low-impact workflows.
- Fallback may be legally required, but it still needs testing and limits.

## Remediation Patterns

- Set per-workflow deadlines and dependency timeouts.
- Make side-effecting operations idempotent before retrying.
- Bound retry attempts and use exponential backoff with jitter.
- Limit concurrency and queues; add admission control and load shedding.
- Scope failure containment to the affected dependency, shard, tenant, or operation.
- Separate liveness, readiness, and startup checks.
- Exercise fallback/failover regularly or remove harmful fallback.
- Test recovery paths with fault injection, replay, rollback, and outage scenarios.

## Good Finding Example

Title: Invoice job can duplicate charges when the payment provider times out

Evidence summary: The invoice worker retries `chargeCustomer()` three times after any timeout but does not pass an idempotency key or persist a processed-charge record. If the provider processes the first request but the response times out, the retry can create a second charge.

Severity: critical for real payments; high for lower-value side effects.

Confidence: high when provider idempotency is absent in the traced call.

## Weak Or Unacceptable Finding Example

"Add circuit breakers for reliability."

Reject this unless the reviewer shows a dependency failure mode, current behavior, resource impact, and why a breaker or other containment mechanism is the right remediation.

## Source Summary

The first-pass reliability lens is grounded in Google SRE for SLOs, overload, cascading failure, reliability testing, launches, and postmortems; AWS Well-Architected and AWS Builders Library for timeouts, retries, idempotency, load shedding, and fallback; Microsoft architecture patterns for retry/circuit breaker guidance; Kubernetes probes for health semantics; IETF RFC 9110 for HTTP idempotency; and OpenTelemetry for reliability evidence.

