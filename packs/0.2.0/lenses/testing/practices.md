# Testing Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The testing lens reviews whether tests, evals, and verification artifacts provide trustworthy regression protection for the system's important risks. It focuses on coverage quality, test level selection, negative and boundary coverage, integration fidelity, contracts, flakiness, assertion strength, security/reliability/data failure modes, fuzz/property tests, and AI evals.

It should not file generic "add tests" findings. Every testing finding should identify a behavior, risk, current verification gap, and plausible escaped regression.

## Subtopic Taxonomy

- Risk-based coverage: critical workflows, business impact, safety/security/data risk.
- Test levels: unit, component, integration, contract, API, UI, E2E, acceptance, exploratory.
- Test types: functional, regression, security, reliability, performance, accessibility, compatibility, ML evals.
- Negative and boundary testing: invalid input, permission denial, timeouts, cancellations, duplicates, edge values.
- Test pyramid and feedback: fast lower-level tests, focused integration tests, limited high-value E2E.
- Integration fidelity: real dependencies, verified fakes, containers, emulators.
- Contract testing: consumer expectations, provider verification, schema compatibility.
- Flakiness: nondeterminism, shared state, time, order, sleeps, network, parallelism.
- Assertion quality: meaningful oracles, mutation testing, snapshots, mocks.
- Generative testing: property-based tests, fuzzing, parser/input surfaces.
- AI evals: task, regression, safety, adversarial, retrieval, tool, model/prompt gates.

## High-Value Review Questions

- Which critical behavior would not be caught if it regressed?
- Are tests at the cheapest level that proves the risk?
- Do negative and failure-path tests assert state preservation?
- Are integration tests using dependency semantics close enough to production?
- Are cross-service/client contracts executable?
- Are tests trusted, or does flakiness make CI noise?
- Would tests fail if the important behavior were removed?
- Are security, reliability, data, and AI failure modes covered by targeted tests/evals?

## Concrete Signals

- Critical workflow has no test or only helper-level tests.
- Happy-path-only tests around state transitions, authz, migrations, or retries.
- E2E-heavy suite with slow/flaky CI and little lower-level coverage.
- In-memory DB or fake queue hides production transaction/constraint/retry behavior.
- API schema changes lack consumer/provider contract tests.
- Tests use sleeps and shared global state.
- Snapshot tests update broad output without semantic assertions.
- Prompt/model changes lack repeatable eval gates.

## Anti-Patterns

- Coverage percentage as quality proof.
- More E2E tests as default answer to every risk.
- Mocking every collaborator and never testing real boundaries.
- Permanent quarantine of flaky tests.
- Tests that assert no exception but not outcome.
- Golden files updated automatically.
- Demo prompts/examples treated as AI evals.

## Evidence Requirements

Testing findings need:

- behavior or risk that should be protected;
- current tests/evals and why they are insufficient;
- escaped regression scenario;
- file paths for code and tests where possible;
- confidence caveats for external CI, staging, eval, compliance, or manual suites;
- remediation specifying test type and level.

## Severity Guidance

- `critical`: unverified critical behavior can cause safety, financial, legal, security, privacy, destructive, or severe AI/automation harm.
- `high`: core workflow or high-risk failure mode can regress without detection.
- `medium`: important behavior has weak, flaky, or indirect verification.
- `low`: low-impact test quality or coverage gap.
- `info`: testing improvement without current risk.

## Confidence Guidance

- `high`: repository evidence directly shows missing/weak tests for the risk.
- `medium`: external test, eval, staging, or compliance suite may cover it.
- `low`: risk or intended behavior is uncertain.

## False-Positive Guidance

- Some verification may live in another repo, CI system, vendor platform, or compliance process.
- Small prototypes may intentionally have lighter testing.
- A few broad E2E tests can be right for small systems.
- Fakes may be verified against production behavior.
- Snapshot tests can be useful when reviewed carefully and paired with semantic assertions.

## Remediation Patterns

- Add focused regression tests for critical workflows.
- Add table-driven negative/boundary tests.
- Move tests to the lowest level that proves the behavior.
- Add integration tests with real dependencies or verified fakes.
- Add contract tests for independent clients/providers.
- Fix flakiness root causes instead of rerunning until green.
- Strengthen assertions and use mutation testing selectively.
- Add fuzz/property tests for complex input surfaces.
- Add AI evals and release gates for prompt/model/retriever/tool changes.

## Good Finding Example

Title: Cross-tenant authorization has only same-tenant happy-path tests

Evidence summary: `project_access_test.py` verifies that a member can read projects in their tenant but has no negative test for reading another tenant's project by ID. The production handler loads by route ID and the security lens identified object-level authorization as high risk. A cross-tenant regression would pass the current suite.

Severity: high because the untested behavior protects sensitive tenant data.

Confidence: high if no external security suite covers the scenario.

## Weak Or Unacceptable Finding Example

"Increase test coverage."

Reject this. It does not identify behavior, risk, existing test gap, escaped regression, or the right test level.

## Source Summary

The first-pass testing lens is grounded in ISO/IEC/IEEE 29119-1 and ISTQB for testing fundamentals, Fowler/Thoughtworks and Google Testing Blog for test strategy, pytest/Playwright/Testcontainers/Pact/Stryker/Hypothesis/OSS-Fuzz for tooling-specific practices, OWASP WSTG for security testing, Google SRE for reliability testing, and Google ML/Responsible AI sources for ML/AI evals.

