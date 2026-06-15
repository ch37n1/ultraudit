# Final Editor Checklist

Status: batch-1 baseline

Use this checklist before publishing an Ultraudit report.

## Findings

- Every important finding has file/path evidence.
- Every important finding has a concrete scenario.
- Severity is justified by impact and likelihood.
- Confidence is justified separately from severity.
- False-positive risks are acknowledged.
- Recommendations address root cause.
- No raw secrets are reproduced.
- Facts and hypotheses are separated.
- AI claims include use case, eval/control evidence, and user impact.

## Deduplication

- Exact duplicates are merged.
- Same root cause across files is consolidated.
- Cross-lens overlaps have one primary lens.
- Old findings from previous runs are not silently lost.
- Repeated findings preserve history and status.

## Report Quality

- No unsupported high/critical findings.
- No generic best-practice filler.
- No architecture fashion recommendations without local risk.
- No "add tests" findings without critical behavior or regression risk.
- No privacy/security claims without affected data class and exposure path.
- Remediation is actionable and scoped.
- Residual uncertainty is clear.

## Source Use

- Source-backed practices are referenced where useful.
- Sources support the practice, not the existence of the local defect.
- Outdated or weak sources are not used for fast-moving claims.

