# Current Task: Deep Research Practice Pack For Ultraudit

You are a long-running research agent with internet access.

Your task is to perform deep, source-backed research for **Ultraudit**, a CLI orchestrator for deep agentic application review.

Current date: 2026-06-15.

## Product Context

Ultraudit is a console utility for deep application audit and review. It does not try to produce a high-quality audit from one large prompt. Instead, it orchestrates many independent agent reviews, preserves their artifacts, normalizes findings, deduplicates results, and produces final human-readable and machine-readable reports.

Ultraudit reviews applications by:

1. collecting repository context;
2. decomposing the repository into domains;
3. running independent lens reviews per domain;
4. running cross-domain review;
5. synthesizing findings per domain;
6. synthesizing findings at system level;
7. comparing with previous runs;
8. running a final editorial and verification pass.

The goal of this task is to create a **practice pack** that can be used by Ultraudit reviewer agents. Do not write a generic article about best practices. Produce actionable, source-backed review practices that can be embedded into prompts, checklists, synthesis rules, severity models, evidence models, and finding validation.

## Core Requirement

Depth is more important than breadth.

Do **not** create one shallow global source map and stop there. For each review lens, perform a separate deep research cycle with its own source map, source triage, coverage matrix, practice extraction, gap analysis, follow-up research, and final lens practice document.

If full coverage of all lenses is impossible in one run, prefer producing fewer complete lens packs over shallow coverage of every lens. Mark unfinished lenses explicitly.

## Review Lenses

Research these Ultraudit lenses:

1. architecture
2. code-quality
3. security
4. correctness
5. testing
6. reliability
7. performance
8. observability
9. operations
10. api-contracts
11. data-integrity
12. privacy-compliance
13. dependency-supply-chain
14. ux-product
15. ml-ai

Also produce stack-specific and application-archetype-specific practice material where useful. These tracks are subordinate to the lenses: they exist to prevent abstract, one-size-fits-all advice.

- general application review
- Rust
- CLI tools
- async/concurrent systems
- backend APIs
- web frontend
- mobile applications
- desktop applications
- databases and migrations
- distributed systems
- AI/RAG/agentic systems
- deployment and operations

## Coverage Guardrails, Not Priorities

The following topics are not the main themes of the whole research and must not displace the full lens-by-lens work. They are guardrails: known areas that must not be missed while every lens still receives deep coverage.

The expected shape is:

```text
lens -> subtopics -> application archetype variations -> practice atoms -> prompt guidance
```

1. **Domain-Driven Design as an architecture sub-optic**
   - Research DDD as part of the `architecture` lens, not as a replacement for architecture research overall.
   - Cover bounded contexts, ubiquitous language, aggregates, entities, value objects, domain services, repositories, application services, domain events, anti-corruption layers, context mapping, transaction boundaries, modular monoliths, microservices boundaries, and common DDD failure modes.
   - Pay attention to how DDD practices translate into code review signals: misplaced business logic, anemic domain models, leaky infrastructure concerns, unclear ownership, invalid aggregate boundaries, cross-context coupling, and inconsistent domain language.

2. **Application archetype variations**
   - For each lens, consider whether the practice changes materially across application types.
   - At minimum, check web applications, backend services/APIs, mobile applications, CLI tools, desktop applications, data-intensive systems, and ML/AI systems.
   - Do not force every archetype into every lens if it is not relevant. When it is relevant, capture the variation explicitly.

3. **Web and backend practice depth**
   - Web development and backend development need dedicated source-backed material because their review signals differ from generic engineering practice.
   - Web coverage should include frontend architecture, component boundaries, state management, accessibility, browser security, client/server contracts, performance, error states, empty states, form behavior, progressive enhancement where relevant, build tooling, and test strategy.
   - Backend coverage should include API design, service boundaries, authentication and authorization, input validation, persistence, transactions, concurrency, background jobs, migrations, observability, reliability, deployment, and operational failure modes.

4. **ML / AI system practice depth**
   - Research ML, RAG, and agentic system review as both an application-archetype track and part of the `ml-ai` lens.
   - Cover evals, dataset quality, data leakage, prompt injection, tool safety, retrieval quality, hallucination risk, fallback behavior, cost/latency budgets, PII exposure, reproducibility, monitoring, drift, human approval gates, and vendor/model dependency risks.

## Output Directory

Create the following output structure in the repository:

```text
research/
  plan.md
  source-rubric.md
  progress.md
  lenses/
    architecture/
      source-map.md
      source-map.yaml
      coverage-matrix.md
      practice-atoms.yaml
      practices.md
      prompt-guidance.md
      research-gaps.md
    code-quality/
      ...
    security/
      ...
  stacks/
    general.md
    rust.md
    cli.md
    async-concurrent.md
    backend-api.md
    web-frontend.md
    mobile-apps.md
    desktop-apps.md
    database.md
    distributed-systems.md
    ai-rag-agents.md
    operations.md
  architecture-patterns/
    domain-driven-design.md
  archetypes/
    web-applications.md
    backend-services.md
    mobile-applications.md
    cli-tools.md
    desktop-applications.md
    data-intensive-systems.md
    ml-ai-systems.md
  integration/
    lens-boundaries.md
    severity-model.md
    confidence-model.md
    evidence-model.md
    deduplication-rules.md
    final-editor-checklist.md
    final-gaps.md
  prompts/
    base-reviewer-guidance.md
    lens-review-template.md
    domain-synthesis-template.md
    system-synthesis-template.md
    previous-runs-comparison-template.md
    final-editor-template.md
```

If you cannot complete every file, keep `progress.md` accurate and explicit.

## Source Quality Rubric

Classify every source:

- `primary-standard`: standards, specifications, regulatory or security frameworks, official best-practice guides.
- `official-docs`: official language, framework, platform, vendor, or tool documentation.
- `authoritative-book-or-paper`: high-quality book, academic paper, or widely cited technical report.
- `engineering-blog`: high-quality engineering blog from an experienced team or vendor.
- `tooling-docs`: documentation for linters, scanners, profilers, observability tools, CI/CD tools, or dependency tools.
- `case-study`: incident report, postmortem, vulnerability write-up, migration story, or production failure analysis.
- `community`: forum, personal blog, Q&A, or community guide.
- `speculative`: useful idea with weak source backing.

Score every source:

```yaml
url: https://example.com
title: Example title
publisher: Example publisher
published_or_updated: "2025-04-10"
accessed: "2026-06-15"
source_type: official-docs
trust: high
freshness: current
lenses:
  - reliability
subtopics:
  - timeouts
  - retries
notes: Short note on why this source matters.
limitations: Any caveats, missing context, vendor bias, or outdated assumptions.
```

## Freshness Guidance

Prefer current sources when the field changes quickly.

Use these freshness targets:

- `ml-ai`: prioritize sources from the last 12-18 months, while also including stable security/evaluation references.
- `dependency-supply-chain`: prioritize sources from the last 24 months, plus stable standards and ecosystem docs.
- `security`: prioritize sources from the last 24-36 months, plus stable OWASP/CWE/NIST references.
- `observability`, `operations`, `platform`, `deployment`: prioritize sources from the last 24-36 months.
- `performance`: include current framework/runtime guidance plus stable systems references.
- `architecture`, `code-quality`, `testing`: classics are acceptable, but check whether recommendations are still valid for modern tooling and workflows.

Do not discard older sources just because they are old if they are still canonical. Mark them as `classic` or `stable` and explain why they still apply.

## Lens Research Loop

For each lens, run this full sub-flow.

### 1. Define Lens Scope

Create a short scope statement:

- what this lens is responsible for;
- what defect classes it should find;
- what it should not own;
- which neighboring lenses commonly overlap with it.

### 2. Split Lens Into Subtopics

Create a subtopic taxonomy before collecting final practices.

For each lens, also identify application-archetype variations where the review practice changes. Avoid purely abstract engineering advice. For example, testing, security, performance, observability, and UX/product review look different for web applications, backend services, mobile applications, CLI tools, desktop applications, and ML/AI systems.

For the `architecture` lens, Domain-Driven Design must be included explicitly as one important subtopic cluster. Do not reduce DDD to a short mention, but also do not let it replace broader architecture topics such as dependency direction, modularity, coupling, ownership, extensibility, layering, integration boundaries, and deployment architecture.

Example for security:

- authentication
- authorization
- session lifecycle
- input validation
- injection
- secrets
- cryptography
- trust boundaries
- supply-chain interaction
- logging and data exposure
- secure defaults

Each lens needs its own subtopic taxonomy.

### 3. Source Discovery In Waves

Do source discovery in at least three waves:

1. primary standards and official docs;
2. tooling docs, framework docs, and authoritative engineering material;
3. case studies, recent articles, postmortems, and targeted gap-filling sources.

Do not stop after the first useful sources. Explicitly search for counterexamples, false-positive risks, and operational failure modes.

### 4. Source Quotas

For each major lens, aim for:

- `30-80` discovered sources;
- at least `15-30` accepted sources after triage;
- at least `6-10` primary, official, standards, or authoritative sources;
- at least `5` recent sources if the field changes quickly;
- at least `3` tooling or framework documentation sources where applicable;
- at least `2` case studies, incident reports, vulnerability write-ups, or postmortems where applicable;
- an explicit list of rejected sources with rejection reasons.

These are targets, not arbitrary padding requirements. Do not include low-quality sources merely to hit a number. If a quota is not appropriate for a lens, explain why.

### 5. Triage Sources

For every accepted source, record:

- source type;
- trust level;
- freshness;
- subtopics covered;
- concrete practices extracted;
- limitations.

For rejected sources, record:

- URL/title;
- reason for rejection;
- whether it may still be useful as background.

### 6. Extract Practice Atoms

Each practice atom must be structured and reviewable.

Use this schema:

```yaml
id: security-authz-object-level-access
title: Object-level authorization must be checked near protected resource access
lens: security
subtopics:
  - authorization
applies_to:
  - backend-api
  - web-app
  - multi-tenant-system
defect_classes:
  - broken-access-control
review_questions:
  - Can a user access, modify, or delete a resource owned by another user, account, tenant, or organization?
signals:
  - Resource is loaded by user-controlled identifier without an adjacent authorization check.
  - Authorization depends only on authentication or role, not on object ownership or policy.
anti_patterns:
  - Checking only that the requester is logged in before returning tenant-scoped data.
evidence_required:
  - File path and line range where resource access happens.
  - File path and line range showing missing, insufficient, or bypassable authorization.
  - Realistic misuse or exploit scenario.
severity_guidance:
  critical: Direct cross-tenant data access or privileged mutation is reachable.
  high: Unauthorized access to sensitive user or business data is plausible.
  medium: Authorization is indirect, incomplete, or fragile but exploitability is uncertain.
  low: Defense-in-depth issue or unclear policy expression with limited demonstrated impact.
confidence_guidance:
  high: The control flow and missing check are directly evidenced.
  medium: The issue is likely but depends on framework or middleware behavior.
  low: The finding is a hypothesis requiring confirmation.
false_positive_risks:
  - Authorization may be enforced in middleware, database row-level security, generated code, or framework policy hooks.
remediation_patterns:
  - Centralize authorization policy while keeping object identity, actor identity, and action explicit.
  - Add negative authorization tests for cross-tenant and cross-owner access.
sources:
  - url: https://example.com
    source_type: primary-standard
    trust: high
prompt_guidance: >
  Trace actor identity, resource identity, action, policy decision, and enforcement point together.
```

Do not create practice atoms that cannot lead to evidence-backed findings.

### 7. Build Lens Practice Document

For each lens, write `practices.md` with:

- lens scope;
- subtopic taxonomy;
- high-value review questions;
- concrete signals reviewers should search for;
- anti-patterns;
- evidence requirements;
- severity guidance;
- confidence guidance;
- false-positive guidance;
- remediation patterns;
- examples of good findings;
- examples of weak or unacceptable findings;
- source summary.

### 8. Build Prompt Guidance

For each lens, write `prompt-guidance.md` containing reviewer-agent instructions. It should be usable inside an Ultraudit prompt.

Include:

- role framing;
- what to inspect first;
- how to follow evidence;
- what to ignore;
- how to handle uncertainty;
- required output fields;
- common mistakes to avoid.

### 9. Coverage Matrix

Before finalizing each lens, create `coverage-matrix.md`.

The matrix must show subtopic coverage by source type:

```text
Subtopic                 Standards  Official docs  Tooling docs  Recent sources  Case studies  Status
Access control           ok         ok             ok            weak            weak          needs follow-up
Secrets handling         ok         ok             ok            ok              missing       needs follow-up
Input validation         ok         ok             weak          ok              weak          acceptable
```

Allowed statuses:

- `strong`
- `acceptable`
- `weak`
- `missing`
- `needs follow-up`

If any major subtopic is `weak`, `missing`, or `needs follow-up`, perform targeted follow-up searches before finalizing that lens. If it remains weak, explain why in `research-gaps.md`.

### 10. Lens Completion Gate

A lens is not complete until:

- every major subtopic has source coverage;
- every important practice has at least one credible source;
- severity guidance exists;
- confidence guidance exists;
- evidence requirements exist;
- false-positive guidance exists;
- prompt guidance exists;
- recent sources were checked where the field changes quickly;
- weak areas are listed explicitly in `research-gaps.md`.

Additional mandatory completion requirements:

- The `architecture` lens is not complete until Domain-Driven Design has dedicated source coverage, practice atoms, review questions, anti-patterns, and prompt guidance, while still preserving broad architecture coverage.
- The stack/archetype-specific material is not complete until web applications, backend services, mobile applications, CLI tools, and ML/AI systems each have dedicated source-backed practice notes where their review signals differ from generic practice.
- The `ml-ai` lens is not complete until it covers both classical ML system review concerns and modern LLM/RAG/agentic-system concerns.

## Integration Pass

After completing lens-level research, run an integration pass. Do not redo all research. Instead, normalize and connect the lens outputs.

Create:

### `integration/lens-boundaries.md`

Define:

- primary responsibility of each lens;
- common overlaps;
- primary lens vs secondary tags;
- examples of where a finding belongs.

Examples:

- Broken object-level authorization is primarily `security`, with possible `api-contracts` or `data-integrity` tags.
- Retry without idempotency may be primarily `reliability`, with `correctness` and `data-integrity` tags.
- PII in logs may be primarily `privacy-compliance`, with `observability` as a secondary tag.

### `integration/severity-model.md`

Create a cross-lens severity model:

- critical
- high
- medium
- low
- info

Define severity in terms of:

- user impact;
- security/privacy impact;
- data loss or corruption;
- availability impact;
- business impact;
- operational blast radius;
- likelihood;
- reversibility;
- evidence strength.

### `integration/confidence-model.md`

Define:

- high confidence;
- medium confidence;
- low confidence;
- hypothesis;
- needs verification;
- likely false positive.

Explain how confidence differs from severity.

### `integration/evidence-model.md`

Define minimum evidence requirements for accepted findings:

- file paths;
- line ranges when available;
- execution path or data flow;
- concrete scenario;
- impact;
- recommendation;
- uncertainty;
- source-backed practice reference when relevant.

Also define what should be rejected or downgraded:

- generic advice;
- unsupported claims;
- style preferences without maintainability impact;
- findings with no domain connection;
- findings with no concrete evidence.

### `integration/deduplication-rules.md`

Define how to deduplicate:

- exact duplicate findings;
- same root cause across multiple files;
- same symptom with different root causes;
- cross-lens overlap;
- findings repeated across runs;
- speculative duplicates.

### `integration/final-editor-checklist.md`

Create a final report quality checklist:

- no unsupported high-severity findings;
- no duplicate findings;
- facts separated from hypotheses;
- evidence present for every important finding;
- severity justified;
- confidence justified;
- remediation actionable;
- old findings from previous runs not silently lost;
- source-backed practices referenced where useful.

### `integration/final-gaps.md`

List:

- weakly researched lenses;
- weakly researched subtopics;
- missing primary sources;
- areas needing specialist review;
- areas where practices change quickly and should be refreshed often.

## Prompt Templates

Create reusable prompt templates for Ultraudit:

### `prompts/base-reviewer-guidance.md`

General reviewer behavior:

- evidence-first;
- domain-focused;
- no generic advice;
- no hidden chain-of-thought required;
- provide concise reasoning summaries;
- distinguish facts from hypotheses;
- mark uncertainty;
- produce structured findings.

### `prompts/lens-review-template.md`

Template for reviewing one domain through one lens.

It must include placeholders:

- `{{ domain_id }}`
- `{{ domain_name }}`
- `{{ lens_id }}`
- `{{ lens_practices }}`
- `{{ repository_context }}`
- `{{ domain_context }}`
- `{{ output_paths }}`

### `prompts/domain-synthesis-template.md`

Template for merging findings across lenses for one domain.

### `prompts/system-synthesis-template.md`

Template for system-level synthesis across all domains.

### `prompts/previous-runs-comparison-template.md`

Template for comparing current findings with previous Ultraudit runs.

### `prompts/final-editor-template.md`

Template for final quality verification of the report.

## Finding Quality Contract

Ultraudit findings should support this structure:

```yaml
id: security-auth-001
title: Session validation bypass in refresh flow
domain: auth
lens: security
severity: high
confidence: medium
status: open
evidence:
  - file: crates/auth/src/session.rs
    lines: "120-155"
    note: Expiry is checked after session rotation.
impact: Expired refresh tokens may be accepted in some paths.
recommendation: Validate token expiry before rotating the session.
effort: medium
tags:
  - auth
  - session
  - token-lifecycle
related_findings: []
practice_refs:
  - security-session-token-lifecycle
```

Your research should make this contract easier to satisfy.

## Working Rules

- Work iteratively and keep `research/progress.md` current.
- Do not rely on memory when a current source is needed.
- Prefer primary and official sources.
- Use secondary sources to enrich practice extraction, not as the only basis for major claims.
- Cite sources for major recommendations.
- Keep source URLs and dates.
- Mark source limitations.
- Avoid vague practices such as "write clean code" or "add tests" unless converted into specific reviewable signals.
- Do not request or expose hidden chain-of-thought. Provide concise reasoning summaries and evidence trails instead.
- Preserve rejected-source notes; they are useful for later review.
- If sources conflict, document the conflict and explain the chosen synthesis.
- If a recommendation depends on context, state the context.
- If a lens is unfinished, mark it unfinished rather than pretending it is complete.

## Final Deliverable

At the end, provide:

1. a concise summary of completed lenses;
2. a list of incomplete or weak areas;
3. the most important source-backed principles discovered;
4. the strongest prompt guidance patterns for reviewer agents;
5. the biggest risks for Ultraudit if these practices are used poorly;
6. exact paths to the generated files.
