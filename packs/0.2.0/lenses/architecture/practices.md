# Architecture Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The architecture lens reviews whether the system's structure fits its domain, quality goals, deployment shape, and change pressures. It finds structural risks: unclear ownership, boundary erosion, invalid service decomposition, dependency inversion failures, missing decision rationale, hidden runtime dependencies, DDD modeling failures, and AI/agent authority-boundary gaps.

It should not become a generic code-quality or style lens. Local complexity belongs primarily to code-quality. Missing tests belong to testing. Auth bypass belongs to security unless the root cause is a trust-boundary design failure. N+1 queries belong to performance unless the root cause is cross-boundary data ownership or API composition.

## Subtopic Taxonomy

- Architecture description: stakeholders, concerns, views, rationale, correspondences.
- Boundaries and ownership: domains, modules, services, adapters, data stores, model providers.
- Dependency direction: core policy separated from delivery, persistence, vendor, and infrastructure mechanisms.
- Quality attribute tradeoffs: modifiability, reliability, security, performance, operability, usability, cost, sustainability.
- Runtime and deployment views: deployable units, workers, queues, jobs, backing services, configuration, environments.
- Integration style: synchronous calls, events, queues, APIs, shared databases, published languages, anti-corruption layers.
- Data ownership: write ownership, read models, materialized views, migrations, cross-boundary state.
- Domain-Driven Design: ubiquitous language, bounded contexts, context maps, aggregates, entities, value objects, domain services, repositories, application services, domain events, transaction boundaries.
- Legacy and migration: strangler patterns, bubble contexts, anti-corruption, coexistence, rollback.
- AI/RAG/agentic architecture: prompt assembly, retrieval boundaries, tool authority, provider dependency, model fallback, evaluation gates.

## High-Value Review Questions

- What are the major domains or capabilities, and where is each one owned?
- Can the dependency graph be explained without circular or hidden ownership?
- Which decisions are hard to reverse, and is their rationale visible?
- What quality scenarios justify the architecture's complexity?
- Where do domain rules live, and can alternate entrypoints bypass them?
- Which data stores, tables, indexes, vector stores, or streams have multiple writers?
- Are distributed boundaries independent, or do they create a distributed monolith?
- Are runtime dependencies, workers, jobs, queues, migrations, and model providers visible?
- For AI systems, what can the model or agent observe, decide, and do?

## Concrete Signals

- Cross-domain imports bypass public interfaces.
- Framework handlers, UI components, or CLI commands contain core business rules.
- Domain entities are data bags while application services implement all rules.
- Same model or table is reused by contexts with different terminology or invariants.
- Multiple services write the same operational data.
- Services deploy in lockstep or require shared database migrations.
- Architecture docs show source folders but omit runtime/deployment behavior.
- Quality claims such as scalable, reliable, secure, or maintainable lack measurable scenarios.
- LLM outputs trigger tools or mutations without typed authorization and approval boundaries.

## Anti-Patterns

- Technology-first decomposition: folders or services by framework layer rather than business responsibility.
- Distributed monolith: network-separated components with lockstep deploys, shared data, and synchronous chains.
- Big shared model: one canonical domain model for terms that have different meanings across contexts.
- Anemic domain model in complex domains: data objects plus service scripts, with invariants spread across use cases.
- Architecture by slogan: microservices, serverless, CQRS, DDD, or event-driven design without a quality goal.
- Diagrams without decisions: static pictures that omit rationale, risks, runtime behavior, and tradeoffs.
- AI ambient authority: model or agent has broad data/tool access without enforceable boundaries.

## Evidence Requirements

Architecture findings need more than preference. Require:

- file paths and line ranges for dependency, rule placement, write path, or runtime coupling evidence;
- docs/config paths for missing or stale architecture artifacts when relevant;
- an execution, change, deployment, failure, or misuse scenario;
- the violated boundary, quality scenario, invariant, or decision rationale;
- impact and blast radius;
- false-positive checks, especially for external docs, generated code, framework behavior, or intentionally simple systems.

## Severity Guidance

- `critical`: structural issue enables cross-tenant access, data corruption, unsafe AI action, total outage, or irreversible production/regulatory harm.
- `high`: core domain, security, data, or availability behavior is likely wrong or unsafe because of the architecture.
- `medium`: architecture materially increases change, deploy, operations, or correctness risk for important flows.
- `low`: localized structural issue with limited blast radius or mainly documentation/rationale risk.
- `info`: useful architecture improvement without demonstrated risk.

## Confidence Guidance

- `high`: code/config/docs directly show the dependency path, boundary violation, invariant split, runtime omission, or unsafe authority path.
- `medium`: evidence is strong but depends on intended ownership, framework behavior, or external artifacts.
- `low`: plausible architecture smell with incomplete domain or runtime context.
- `hypothesis`: preserve for synthesis notes, not accepted as a finding unless the reviewer identifies verification steps.

## False-Positive Guidance

- Do not demand DDD, microservices, hexagonal architecture, ADRs, or C4 diagrams for every project.
- Simple CRUD systems can use transaction scripts safely when business rules are minimal.
- Monoliths may share one database internally without violating service ownership.
- External issue trackers, IaC repos, compliance docs, or platform gateways may contain missing rationale or controls.
- Functional or data-oriented designs can keep domain behavior in pure functions rather than entity methods.

## Remediation Patterns

- Define public interfaces and dependency direction at important boundaries.
- Move core policy out of delivery/infrastructure code where bypass or duplication risk exists.
- Add lightweight ADRs for high-cost or hard-to-reverse decisions.
- Write quality attribute scenarios for major architecture claims.
- Introduce context maps, anti-corruption layers, and explicit translations for DDD boundaries.
- Assign data owners and route mutations through owner modules or APIs.
- Prefer modular monolith boundaries when distribution is not justified.
- For AI systems, separate instructions, data, retrieved content, model output, tool calls, and approval gates.

## Good Finding Example

Title: Billing and subscription contexts share one `Customer` model with incompatible lifecycle rules

Evidence summary: `billing/customer.py` treats suspended customers as billable until invoice close, while `subscriptions/customer.py` treats suspension as immediate access revocation. Both import and mutate `common/customer.py`, so a change for subscription enforcement can alter billing eligibility. No context map or translation layer exists.

Severity: high, because a core revenue/access invariant can be violated across contexts.

Confidence: high, because the conflicting rules and shared mutation path are directly evidenced.

## Weak Or Unacceptable Finding Example

"The project should use DDD and microservices because that is better architecture."

Reject this. It has no domain evidence, no concrete defect class, no source path, no user impact, and no proof that distribution or DDD would improve the system.

## Source Summary

The architecture lens is grounded in ISO 42010 for architecture descriptions, SEI ATAM for scenario-driven tradeoff analysis, arc42 and C4 for practical views, AWS Well-Architected and Google SRE for quality-oriented review, Domain Language and Fowler for DDD, Microsoft and Microservices.io for service/DDDesign patterns, and NIST/OWASP sources for secure and AI-aware architecture.

