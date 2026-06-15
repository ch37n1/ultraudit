# Correctness Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The correctness lens reviews whether behavior matches intended product, domain, protocol, and runtime semantics. It covers invariants, state transitions, concurrency histories, edge cases, negative paths, retry semantics, time/date handling, Unicode/locale behavior, numeric calculations, UI event ordering, and ML/data-pipeline behavior.

Correctness differs from data-integrity because it also includes non-persistent and user-visible behavior. It differs from testing because it identifies the defect; testing evaluates whether the defect class is covered.

## Subtopic Taxonomy

- Invariants: rules that must always hold.
- State machines: legal states, transitions, terminal states, retries, cancellation, partial completion.
- Concurrency: races, stale reads, lost updates, write skew, cache/state histories.
- Idempotency and duplicate intent: retries, double-submit, duplicate delivery, unknown outcomes.
- Negative paths: validation failures, dependency failures, exceptions, cancellation, timeouts.
- Time and calendar semantics: instants, local dates/times, durations, recurrences, time zones, DST, tzdb updates.
- Unicode and locale semantics: normalization, collation, case folding, identifiers, search.
- Numeric semantics: money, counts, measurements, rounding, overflow, precision, units.
- Protocol semantics: HTTP safe/idempotent methods, status meanings, client/server assumptions.
- Event-driven UI: event ordering, stale async responses, optimistic UI, cancellation.
- ML/AI correctness: feature skew, metric mismatch, eval gaps, RAG grounding, hallucination controls.

## High-Value Review Questions

- What must always be true before and after this operation?
- Which states can this workflow be in, and which transitions are legal?
- Can two actors or two events interleave to produce an invalid result?
- Does retry mean "same intent" or "new intent"?
- What happens after timeout, cancellation, or unknown outcome?
- Does the code distinguish instant, local date, local time, duration, and recurrence?
- What equality/order semantics are intended for text?
- Are numeric units, scale, rounding, and overflow explicit?
- Can stale async UI responses override newer intent?

## Concrete Signals

- Business rules duplicated with different conditions.
- Boolean flags encode impossible state combinations.
- Check-then-act around scarce resources.
- Retried mutation lacks durable idempotency semantics.
- Catch-all error handler returns success or default state.
- Future local schedule stored only as UTC offset.
- User identity/search uses raw byte comparison for human text.
- Money calculated with binary floating-point.
- Async UI request responses are applied without request identity.

## Anti-Patterns

- Happy-path behavior treated as full correctness.
- Single-threaded test used as proof against races.
- UI/client validation as domain correctness.
- Default locale/time zone used in domain logic.
- Floating-point exact equality for business decisions.
- Retrying unknown outcomes without preserving intent.
- "Last response wins" in interactive UIs.

## Evidence Requirements

Correctness findings need:

- intended behavior, invariant, protocol semantic, or domain rule;
- code path and line ranges when possible;
- concrete input, state, event sequence, or interleaving;
- actual wrong result;
- affected user/data/workflow;
- false-positive checks for external constraints, framework behavior, product intent, and runtime configuration.

## Severity Guidance

- `critical`: wrong behavior can cause safety, legal, financial, access, cross-tenant, destructive, or irreversible harm.
- `high`: core workflow can produce materially wrong user/business outcome.
- `medium`: important but recoverable behavior is wrong under realistic edge/concurrent/negative conditions.
- `low`: localized or low-impact correctness issue.
- `info`: unclear or low-risk correctness improvement.

## Confidence Guidance

- `high`: intended rule and wrong path are directly evidenced.
- `medium`: behavior is likely wrong but product intent, framework semantics, or external controls need confirmation.
- `low`: suspicious edge case without enough domain confirmation.

## False-Positive Guidance

- Product requirements may intentionally permit behavior that looks odd.
- Eventual consistency may be correct when convergence and user contract are explicit.
- Frameworks, databases, and workflow engines may enforce state, rollback, serialization, or cancellation.
- Approximate numeric behavior may be correct for analytics or ML scores.
- Binary text comparison may be correct for opaque identifiers.

## Remediation Patterns

- Centralize invariants and state transitions.
- Represent state machines explicitly.
- Add concurrency controls or redesign for single-writer/conditional writes.
- Use durable idempotency semantics for retryable mutations.
- Separate time concepts and store zone IDs for local future intent.
- Define text normalization/collation policies per field.
- Use domain-appropriate numeric types and rounding policy.
- Make negative outcomes explicit: success, failure, unknown, cancelled, partial.
- Track async request identity and cancellation in UIs.

## Good Finding Example

Title: Concurrent seat booking can oversell an event

Evidence summary: `reserveSeat()` reads remaining capacity, then inserts a reservation if the count is positive. There is no row lock, unique constraint, conditional update, or serializable retry. Two concurrent requests can both read one remaining seat and both commit reservations.

Severity: high for normal ticketing; critical if the resource controls safety or legal access.

Confidence: high if no database constraint or lock exists on the path.

## Weak Or Unacceptable Finding Example

"This code may have edge cases."

Reject this. It does not identify the edge case, intended behavior, concrete input/state, actual wrong result, or impact.

## Source Summary

The first-pass correctness lens is grounded in TLA+ for invariant/state reasoning, Jepsen for consistency histories, PostgreSQL for transaction anomalies and constraints, AWS idempotency for retry semantics, IETF HTTP/date-time RFCs and IANA tzdb for protocol/time correctness, Unicode UAX/UTS for text equivalence and ordering, IEEE 754 for numeric behavior, W3C UI Events for browser event semantics, Google SRE for data correctness, and Google Rules of ML for ML pipeline correctness.

