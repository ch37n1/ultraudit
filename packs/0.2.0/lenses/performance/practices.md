# Performance Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The performance lens reviews whether the application meets realistic latency, throughput, resource, startup, cost, and user-experience needs. It covers service latency, database access, network round trips, caching, async blocking, contention, frontend rendering, mobile/device constraints, telemetry evidence, and AI/model cost and latency.

Performance findings should not be taste-based. They need a workflow, resource, growth behavior, budget, measurement, or plausible production data/device scenario.

## Subtopic Taxonomy

- Budgets and objectives: latency, throughput, tail latency, startup, memory, CPU, cost, user experience.
- Algorithms and growth: complexity, unbounded loops, input/data-size behavior.
- Database: N+1, query plans, indexes, pagination, scans, sorts, joins, connection pools.
- Network: waterfalls, payload size, compression, pagination, streaming, synchronous chains.
- Caching: hit rate, keys, invalidation, freshness, stampede, CDN/client/server cache.
- Concurrency: async blocking, lock contention, thread-pool starvation, unbounded tasks.
- Frontend: Core Web Vitals, main-thread work, render-blocking resources, bundle size, hydration, layout stability.
- Mobile/desktop: startup, memory, battery, offline sync, low-end devices.
- AI: token/model/retrieval/tool budget, model selection, context size, latency, provider quotas.
- Measurement: profiling, benchmarks, query plans, field/lab metrics, saturation telemetry.

## High-Value Review Questions

- What user or business outcome is slow, expensive, or resource-heavy?
- Does work grow with input size, row count, result count, request fanout, or agent steps?
- Is there a measurable budget or SLO for the path?
- Does the path perform avoidable sequential network or database calls?
- Are caches safe, observable, and correctly keyed?
- Can blocking work starve an event loop, UI thread, thread pool, or shared lock?
- Are web performance metrics tied to real users and routes?
- Are AI model/retrieval/tool costs bounded?

## Concrete Signals

- Loop performs per-row query or remote call.
- Query plan scans/sorts large tables on critical path.
- Frontend route serially fetches data before rendering useful content.
- Cache key omits tenant, locale, auth, feature flag, model/index version, or parameter.
- Blocking IO or CPU work runs on event loop or UI thread.
- Unbounded task spawn, queue, or agent loop.
- Large JS bundle or render-blocking resource blocks LCP/INP.
- AI prompt sends excessive context or uses expensive model for simple task.

## Anti-Patterns

- Average latency only.
- Works on seed data.
- Optimize without measuring.
- Cache as first fix with no invalidation plan.
- Client-side pagination of unbounded server results.
- Memoization before fixing data flow or render size.
- AI context/model size increased without budget.

## Evidence Requirements

Performance findings need:

- workflow and user/business/operational impact;
- resource being consumed;
- code path or telemetry artifact;
- growth behavior, measurement, or realistic scenario;
- current or inferred budget;
- false-positive checks for caching, indexes, CDN, batching, runtime offload, and external telemetry.

## Severity Guidance

- `critical`: performance issue can cause systemic outage, runaway cost, critical workflow failure, or severe user harm.
- `high`: core workflow is likely too slow, expensive, or resource-heavy at realistic scale.
- `medium`: important path has material performance risk or missing measurement.
- `low`: localized inefficiency with limited impact.
- `info`: optimization opportunity without demonstrated risk.

## Confidence Guidance

- `high`: measured data, query plan, resource trace, or direct complexity evidence supports the finding.
- `medium`: code evidence is strong but production workload/data/device assumptions need confirmation.
- `low`: plausible performance smell with unclear hotness or scale.

## False-Positive Guidance

- Production indexes, caches, CDNs, APM, or service meshes may mitigate.
- Some data sets are bounded by product rules.
- Approximate or expensive AI paths may be acceptable offline.
- Lab web metrics may differ from field data.
- Simpler code may be preferable on cold paths.

## Remediation Patterns

- Define budgets and measure representative workloads.
- Batch/eager-load data and inspect query plans.
- Add indexes or change access pattern when cardinality demands it.
- Reduce sequential network round trips and payload size.
- Use caching with explicit key/freshness/stampede controls.
- Bound concurrency and move blocking work off shared event loops/UI threads.
- Optimize LCP/INP/CLS with field and lab metrics.
- Budget AI tokens, tools, context, model selection, and latency.

## Good Finding Example

Title: Project list endpoint performs one permission query per row

Evidence summary: `listProjects()` loads 100 projects and then calls `canView(project.id)` inside the loop. `canView` executes a database query per project. The endpoint has no explicit page-size cap in the service layer and is on the dashboard load path. Query count grows as `1 + N`.

Severity: high if dashboard is core and production tenants can have hundreds of projects.

Confidence: high when query logging or code path confirms lazy per-row access.

## Weak Or Unacceptable Finding Example

"This could be optimized."

Reject this. It lacks workflow, resource, scale, evidence, and user/business impact.

## Source Summary

The first-pass performance lens is grounded in AWS Performance Efficiency, Google SRE overload/monitoring, AWS latency/retry guidance, web.dev Core Web Vitals, MDN browser performance, Chrome Lighthouse docs, PostgreSQL EXPLAIN, Prometheus/OpenTelemetry instrumentation, React performance docs, and Google ML/AI evaluation sources.

