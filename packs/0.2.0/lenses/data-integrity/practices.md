# Data Integrity Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The data-integrity lens reviews whether data remains correct, complete, non-duplicated, authorized, traceable, recoverable, and consistent across write paths, transactions, migrations, background jobs, distributed messaging, restores, derived stores, search indexes, and RAG/vector indexes.

It should find invalid states, lost updates, duplicate mutations, failed backfills, unsafe migrations, schema drift, stale derived data, untested restores, lost events, inconsistent projections, and index lineage gaps.

## Subtopic Taxonomy

- Invariants and constraints: authoritative enforcement, database constraints, schema validation, aggregate boundaries.
- Transactions and isolation: concurrency anomalies, locking, serialization retry, conditional writes.
- Idempotency: retries, duplicate requests, webhooks, jobs, deduplication.
- Migrations: expand-contract, backfills, compatibility windows, rollback, transaction handling, preconditions.
- Migration history and drift: changelogs, migration tables, reproducible environments, reference data.
- Distributed data consistency: outbox, CDC, sagas, projections, idempotent consumers.
- Backup and restore: RPO/RTO, restore tests, corruption detection, data inventory.
- Derived data: caches, read models, analytics, search indexes, exports, vector indexes.
- Deletion and correction propagation: soft delete, retention, tombstones, privacy overlap.
- AI/RAG data integrity: corpus provenance, source lineage, ACLs, freshness, index rebuilds, grounded citations.

## High-Value Review Questions

- Which data invariants must never be false?
- Where is each invariant enforced, and can any writer bypass it?
- Can two concurrent requests both commit an invalid result?
- Can retries or duplicate delivery create duplicate side effects?
- Can old and new application versions both run during a schema change?
- Are migration files enough to reproduce the current schema and reference data?
- Can a database commit and event publish get out of sync?
- Can backups actually be restored to the level users require?
- Do deletes/corrections propagate to derived stores and RAG/search indexes?
- Can indexed/generated output be traced to source data version and permissions?

## Concrete Signals

- Business invariant enforced only in UI/controller code.
- Check-then-insert without unique constraint or serializable retry.
- Worker/webhook inserts on every delivery with no durable dedup.
- Migration drops column still read by old app, worker, or mobile client.
- Backfill cannot resume safely after partial failure.
- Manual SQL or reference data changes are not versioned.
- Event publish happens outside database transaction.
- Backup exists but restore has never been tested.
- Soft-deleted data remains in search/vector index.
- RAG chunks lack source ID, version, ACL, or ingestion timestamp.

## Anti-Patterns

- Client validation as data integrity.
- Exactly-once assumptions across queues, networks, and brokers.
- Read committed assumed to protect multi-row invariants.
- Destructive migration in same deploy as app cutover.
- Backup success treated as restore proof.
- Derived data called "cache" even when it cannot be deterministically rebuilt.
- RAG index treated as trusted source without lineage and permissions.

## Evidence Requirements

Data-integrity findings need:

- named invariant or data contract;
- affected data class and owner;
- write, migration, transaction, restore, event, or index path;
- concrete concurrency, retry, migration, crash, restore, or propagation scenario;
- local evidence plus caveats for external database/platform controls;
- severity based on data value, reversibility, blast radius, and user impact.

## Severity Guidance

- `critical`: reachable path to unrecoverable data loss, financial/safety/regulatory corruption, cross-tenant corruption, duplicate destructive action, or unauthorized sensitive derived data exposure.
- `high`: core business data can become wrong, stale, duplicated, or unrecoverable with material impact.
- `medium`: important but recoverable inconsistency or migration/restore risk.
- `low`: low-impact data hygiene issue.
- `info`: data quality improvement without current risk.

## Confidence Guidance

- `high`: invariant, path, missing control, and failure scenario are directly evidenced.
- `medium`: external database constraints, triggers, platform backups, migration repos, or data pipelines may provide controls.
- `low`: data ownership or runtime topology is incomplete.

## False-Positive Guidance

- Database constraints, triggers, row-level security, and generated migrations may live outside app code.
- Managed platforms may provide backups, restore tests, idempotency, or deduplication externally.
- Some consistency is intentionally eventual; findings need to show the expected convergence or user-impact gap.
- Derived data may be disposable if deterministic rebuild and source retention are proven.

## Remediation Patterns

- Move critical invariants to authoritative boundaries and database constraints where possible.
- Use proper transaction isolation, conditional writes, locks, or serializable retry for concurrency-sensitive rules.
- Add idempotency keys, unique constraints, and processed-message tables.
- Use expand-migrate-contract and idempotent backfills.
- Version all schema, data, reference, trigger, and stored procedure changes.
- Use transactional outbox or CDC for DB/message atomicity.
- Test restores and verify backup scope includes all user-visible data artifacts.
- Propagate deletes/corrections through derived stores and RAG/search indexes.
- Preserve lineage, ACLs, freshness, and source IDs for indexed AI/search data.

## Good Finding Example

Title: Subscription activation can create two active subscriptions for one account

Evidence summary: `activateSubscription(account_id)` first queries active subscriptions, then inserts a new active row if none exists. The table has no unique partial constraint on active subscription per account, and the transaction uses default read committed isolation. Two concurrent activation requests can both read zero active rows and both insert.

Severity: high if subscription state gates billing or access.

Confidence: high when no unique constraint, lock, or serializable retry exists.

## Weak Or Unacceptable Finding Example

"The app should use transactions."

Reject this unless the reviewer identifies the invariant, current transaction boundary, concurrent/failure scenario, and impact.

## Source Summary

The first-pass data-integrity lens is grounded in Google SRE data integrity, official PostgreSQL/MySQL/SQLite/MongoDB docs, migration guidance from Fowler, Flyway, Liquibase, and Prisma, AWS idempotency guidance, IETF HTTP semantics, Microservices.io transactional outbox, and OWASP RAG security for AI/search corpus integrity.

