## False-Positive Guidance

- Enterprise registries, admission controllers, CI policies, or package mirrors may enforce pins, signatures, and SBOMs outside the repo.
- Libraries can legitimately omit lockfiles when downstream applications own reproducible resolution.
- Vulnerability scanners may over-report unreachable or patched packages.
- Mature stable dependencies can have low activity without being abandoned.
- Legal/commercial approvals may exist outside the repo.

