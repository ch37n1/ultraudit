## False-Positive Guidance

- Database constraints, triggers, row-level security, and generated migrations may live outside app code.
- Managed platforms may provide backups, restore tests, idempotency, or deduplication externally.
- Some consistency is intentionally eventual; findings need to show the expected convergence or user-impact gap.
- Derived data may be disposable if deterministic rebuild and source retention are proven.

