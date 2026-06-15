# ML/AI Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The ML/AI lens reviews whether AI behavior is fit for the product, domain, and operational risk. It covers classical ML systems, ranking/recommendation, generative AI, RAG, and agentic systems. It focuses on data, evals, model behavior, retrieval quality, tool behavior, reproducibility, monitoring, drift, cost/latency, fallback, and human oversight.

Security exploit paths are shared with the security lens. When the primary issue is attacker control, data exfiltration, prompt injection, or unsafe tool authority, tag `security` as primary and `ml-ai` as secondary.

## Subtopic Taxonomy

- Risk framing: use case, user impact, AI lifecycle stage, high-stakes context, system scope.
- Data: provenance, consent, labeling, preprocessing, representativeness, leakage, deletion, retention, datasheets.
- Classical ML engineering: metrics, simple baselines, feature ownership, training/serving skew, silent data failures.
- Model documentation: intended use, limitations, performance by slice, model cards, vendor/model dependencies.
- Evals: task evals, safety evals, regression evals, adversarial evals, red-team evals, external evals, launch gates.
- RAG: corpus quality, chunking, metadata, authorization-aware retrieval, grounding, citations, freshness, relevance.
- LLM behavior: confabulation, uncertainty, refusal/abstention, prompt sensitivity, overreliance.
- Agents: tool permissions, memory, planning loops, approval gates, audit logs, rollback and cancellation.
- Operations: cost budgets, token limits, latency, rate limits, fallbacks, timeouts, retries, monitoring, drift.
- Reproducibility: versioned prompts, models, datasets, indexes, evals, tool schemas, provider settings.

## High-Value Review Questions

- What AI capability is promised, and what exact user decision or workflow depends on it?
- What data or corpus was used, and is its provenance and intended use documented?
- Which evals gate prompt/model/retriever/tool changes?
- Are evals realistic, adversarial, and tied to launch thresholds?
- Can an output be traced to model, prompt, data/index, tool, and eval versions?
- What happens when retrieved context is missing, stale, unauthorized, or conflicting?
- What can an agent do, and where is human approval enforced?
- Are cost, latency, rate-limit, and provider outage failure modes bounded?
- How are drift, unsafe outputs, and feedback-loop failures detected?

## Concrete Signals

- Generic benchmark cited as proof for domain-specific behavior.
- No regression evals for prompt, model, retriever, index, or tool changes.
- RAG answers cite model-generated sources rather than retrieved source IDs.
- RAG retrieval ignores per-user authorization before context assembly.
- Prompt says "always answer" in a factual or high-stakes workflow.
- Model/provider aliases float without version records.
- Vector index rebuilds overwrite prior source lineage.
- Feature code differs between training and serving with no parity tests.
- Agent has broad tool access, no step limit, no approval gate, or weak audit logs.
- Model calls lack timeout, token budget, cost budget, fallback, or retry discipline.

## Anti-Patterns

- Demo-driven AI: shipping because examples look good.
- Benchmark laundering: public benchmark scores replace application evals.
- Vector similarity as truth.
- Prompt text as policy enforcement.
- Model name as capability guarantee.
- User warning as the only mitigation for high-impact false output.
- Automatic feedback-to-training or feedback-to-memory loops without moderation.
- Rebuilding indexes or changing prompts without release artifacts.

## Evidence Requirements

ML/AI findings need:

- AI use case and user/business impact;
- source paths for prompt, model client, retriever, index builder, dataset, eval, tool, or monitoring path;
- missing or insufficient control;
- realistic failure scenario;
- affected users, data, or workflow;
- severity and confidence with external-artifact caveats;
- source-backed practice reference.

## Severity Guidance

- `critical`: AI behavior can cause safety, legal, medical, financial, regulated, production-destructive, privacy, or irreversible harm without adequate controls.
- `high`: core AI feature can mislead users, expose sensitive data, take important action, or regress broadly.
- `medium`: material quality, reliability, cost, eval, provenance, or reproducibility issue in important but recoverable workflows.
- `low`: low-risk AI hygiene gap or internal-only issue.
- `info`: improvement opportunity without current risk.

## Confidence Guidance

- `high`: code/config/eval/data path directly shows the gap.
- `medium`: likely issue but evals, registries, monitoring, or governance may exist outside the repository.
- `low`: suspicious AI risk with incomplete artifact visibility.
- `hypothesis`: use for follow-up notes when a concrete verification path exists.

## False-Positive Guidance

- Evals, model registries, data governance, and monitoring often live outside the application repo.
- Creative/assistive tools may tolerate unsupported output more than factual/high-stakes systems.
- Vendor platforms may enforce model versioning, content filters, logging, quotas, and safety evals externally.
- Internal prototypes need lighter gates than production user-facing AI.
- RAG source display may be unnecessary if output is not factual or decision-supporting.

## Remediation Patterns

- Define risk tier and launch criteria for each AI capability.
- Add task, regression, safety, adversarial, and operational evals.
- Document datasets/corpora with datasheets and models/capabilities with model cards.
- Version prompts, model IDs, provider settings, indexes, datasets, tool schemas, and eval artifacts.
- Add authorization-aware retrieval, source IDs, freshness policies, and grounded citations for factual RAG.
- Add abstention, fallback, human review, and uncertainty behavior for high-impact outputs.
- Constrain agents with least-privilege tools, step limits, typed schemas, approval gates, and audit logs.
- Monitor quality, drift, cost, latency, safety events, retrieval misses, model changes, and feedback loops.

## Good Finding Example

Title: Support-answer RAG can cite stale policy documents after index rebuilds

Evidence summary: `scripts/build_index.ts` rebuilds the vector index from the policy directory but stores only chunk text and embedding IDs, not source document version or policy effective date. The answer path displays generated citations from chunk text. There is no eval or freshness check for replaced policies. Users may receive outdated refund-policy answers after policy changes.

Severity: medium for support guidance; high if the answer is binding or affects regulated obligations.

Confidence: high if no external index metadata is present.

## Weak Or Unacceptable Finding Example

"The app uses an LLM, so it may hallucinate."

Reject this. It lacks use case, output path, user reliance, eval/fallback evidence, and severity basis.

## Source Summary

The first-pass ML/AI lens is grounded in NIST AI RMF and NIST AI 600-1 for risk framing, Google Responsible GenAI Evaluation for eval types, Google Rules of ML and ML Test Score for production ML engineering, Hidden Technical Debt in ML Systems for pipeline and configuration risk, Datasheets and Model Cards for documentation, OWASP LLM/RAG/Agent guidance for modern LLM system risks, and SRE/AWS sources for operational budgets and reliability.

