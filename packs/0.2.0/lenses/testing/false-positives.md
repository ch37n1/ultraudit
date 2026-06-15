## False-Positive Guidance

- Some verification may live in another repo, CI system, vendor platform, or compliance process.
- Small prototypes may intentionally have lighter testing.
- A few broad E2E tests can be right for small systems.
- Fakes may be verified against production behavior.
- Snapshot tests can be useful when reviewed carefully and paired with semantic assertions.

