# Code Quality Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The code-quality lens reviews whether code remains easy to understand, change, review, test, and debug. Findings should explain how the code increases maintenance risk, not merely that it differs from a preference.

## Subtopic Taxonomy

- Maintainability attributes: modularity, analysability, modifiability, reusability, testability.
- Complexity: branching, nesting, data flow, indirection, configuration, genericity, cognitive load.
- Cohesion and decomposition: functions, classes, components, modules, packages, public APIs.
- Duplication: copied logic, repeated conditionals, parallel hierarchies, schema/model repetition.
- Naming and readability: domain terms, misleading names, overloaded abbreviations, hidden units.
- Comments and documentation: why-comments, public API docs, TODO/FIXME debt, stale comments.
- Error handling and resources: exception/result shape, catch-all handlers, cleanup, lifetime ownership.
- Types and contracts: type annotations, nullability, enums, value objects, impossible states.
- Style and tooling: formatter, linter, suppressions, generated-code boundaries, local conventions.
- Refactoring risk: hotspot files, high-churn debt, changes mixed with formatting or unrelated cleanup.

## High-Value Review Questions

- Would a future maintainer understand the code path quickly enough to change it safely?
- Is the complexity necessary for current requirements, or speculative generality?
- Is the code organized around cohesive concepts with clear ownership?
- Are names, types, and comments carrying true domain meaning?
- Is duplication creating multiple places where a rule can drift?
- Do error/resource paths make normal and exceptional control flow clear?
- Are lint suppressions justified and narrow?
- Is generated or vendored code being reviewed as if it were hand-written?
- Is the finding anchored in a maintainability impact, not taste?

## Concrete Signals

- Function/component combines validation, IO, formatting, authorization, and persistence in one hard-to-test block.
- Deeply nested branches encode business states that should be named or decomposed.
- Boolean flag or mode string changes a function into several hidden behaviors.
- Comment explains what unclear code does instead of the code being simplified.
- TODO/FIXME marks a known risky path that is complex, high-churn, or user-critical.
- Lint suppression is broad, unexplained, or hides unrelated warnings.
- Catch-all error handling swallows real failures or makes cleanup ambiguous.
- Frontend component owns unrelated server state, form state, rendering, effects, and layout.
- Massive formatting churn is mixed with behavior change, obscuring review and rollback.

## Anti-Patterns

- Filing style preferences when the project has no matching rule and no maintainability impact.
- Treating a metric threshold as sufficient evidence.
- Demanding large refactors without showing a risky change path.
- Penalizing generated code or framework boilerplate without a hand-written boundary problem.
- Converting every code smell into a finding; smells are prompts for investigation.
- Ignoring local idioms in favor of generic textbook advice.

## Evidence Requirements

Code-quality findings need:

- specific code location and maintainability defect;
- why the current shape increases change, review, test, debug, or defect risk;
- local convention, source-backed practice, or repeated project pattern;
- false-positive checks for generated code, framework requirements, small bounded scope, and deliberate tradeoffs;
- concrete remediation sized to the risk.

## Severity Guidance

- `critical`: maintainability defect blocks safe remediation of critical production, security, compliance, or data-loss risk.
- `high`: core high-churn or business-critical area is hard to modify safely and likely to cause future defects.
- `medium`: important code path has material maintainability risk with plausible near-term change pressure.
- `low`: localized readability, style, duplication, or refactoring issue with limited blast radius.
- `info`: optional polish or educational note with no required change.

## Confidence Guidance

- `high`: code shape, local conventions, churn/usage, and impact are directly evidenced.
- `medium`: maintainability issue is clear but change frequency or ownership is inferred.
- `low`: likely smell with limited context or no demonstrated maintenance pressure.

## False-Positive Guidance

- Small one-off functions can be simple enough despite metric violations.
- Some duplication is intentional to avoid premature abstraction across unstable concepts.
- Frameworks may require boilerplate or naming conventions.
- Generated, vendored, migration, snapshot, fixture, and test-golden files require different review standards.
- Broad compatibility code can be ugly by necessity; judge isolation and tests.

## Remediation Patterns

- Split by responsibility, not line count alone.
- Name intermediate concepts instead of adding comments that repeat code.
- Replace ambiguous flags with explicit functions, types, or command objects.
- Extract duplicated domain rules into one owned implementation when concepts are stable.
- Narrow `try`/`catch` or `Result` handling and make cleanup explicit.
- Add or tighten types so invalid states are harder to express.
- Move formatting-only changes into separate commits or diffs.
- Justify lint suppressions with narrow scope and revisit markers.

## Good Finding Example

Title: `buildInvoicePreview` mixes pricing, permission filtering, formatting, and network retries in one change-prone function

Evidence summary: The function handles authorization filtering, discount calculation, external tax lookup, HTML formatting, retry scheduling, and telemetry in one 180-line block. Recent changes touch the same function for unrelated pricing and UI changes. There are no named seams for the pricing rule or retry behavior, so a future pricing change can silently affect permissions or retries.

Severity: medium or high depending on business criticality and churn.

Confidence: high when the code path, responsibilities, and change history are visible.

## Weak Or Unacceptable Finding Example

"This function is too long."

Reject this unless the finding explains the responsibilities, change risk, and concrete maintainability impact.

## Source Summary

This first-pass lens is grounded in ISO/IEC 25010 maintainability framing, Google Engineering Practices, Google style guides, PEP 8, Effective Go, Go Code Review Comments, Rust API Guidelines, Clippy, rustfmt, ESLint, Pylint, Martin Fowler refactoring and technical-debt material, React component guidance, and empirical maintainability research.
