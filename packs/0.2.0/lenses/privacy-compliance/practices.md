# Privacy Compliance Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The privacy-compliance lens reviews how personal data flows through the system and whether code, configuration, vendors, telemetry, UX, and operations support privacy obligations. Findings should be implementation-specific and avoid pretending to provide legal advice.

## Subtopic Taxonomy

- Data identification: PII, sensitive data, pseudonymous identifiers, device IDs, inferred data, prompt/file content.
- Purpose and lawful basis: collection reason, feature necessity, consent, contract, legal obligation, legitimate interest.
- Minimization and defaults: required vs optional fields, local/on-device processing, least data, privacy-by-default.
- Transparency and consent: notices, consent flows, withdrawal, preference centers, dark-pattern risks.
- Retention and deletion: TTLs, archives, backups, logs, search indexes, data warehouses, caches, ML/RAG indexes.
- Subject rights: access, export, correction, deletion, restriction, objection, portability.
- Telemetry and analytics: logs, traces, crash reports, session replay, product analytics, advertising IDs.
- Vendors and transfers: SDKs, processors, sub-processors, AI vendors, cloud regions, data sharing, DPAs/SCCs.
- DPIA/high-risk processing: profiling, automated decisions, sensitive data, children, large-scale monitoring, AI.
- Mobile and client platforms: runtime permissions, tracking, privacy labels, SDK disclosures, webviews.
- AI-specific privacy: prompts, files, embeddings, vector stores, eval sets, transcripts, provider retention/training defaults.

## High-Value Review Questions

- What personal data is collected, generated, inferred, stored, logged, shared, or sent to vendors?
- What purpose and necessity justify each data element?
- Can the feature work with less precise, less durable, or on-device data?
- Are privacy notices, consent, and store declarations consistent with observed behavior?
- Are retention and deletion implemented across primary stores, backups, caches, logs, indexes, analytics, and AI artifacts?
- Can subject-right requests be fulfilled without manual guesswork?
- Do third-party SDKs, analytics tools, crash reporters, and AI vendors receive data the product team may not realize is sent?
- Does high-risk processing have a DPIA or equivalent privacy-risk record?

## Concrete Signals

- Required signup field is not used for core functionality and has no documented purpose.
- User deletion removes the account row but leaves events, logs, analytics profiles, vector embeddings, or exported files.
- Crash logs, traces, or session replay capture emails, tokens, health data, chat prompts, or form contents.
- Consent banner or permission prompt is bundled, preselected, or not withdrawable.
- Mobile app data safety/privacy label omits collection by bundled SDKs.
- AI feature sends user documents or chat content to a model provider without data-control configuration or notice.
- Retention policy exists in docs but no TTL, purge job, backup policy, or deletion test exists.
- Vendor endpoint or SDK is introduced without inventory, purpose, region, or processor/subprocessor review.

## Anti-Patterns

- "We hash it, so it is not personal data" without re-identification analysis.
- Treating logs, analytics, and embeddings as outside privacy scope.
- Assuming consent covers unrelated future processing.
- Keeping data forever because storage is cheap.
- Deleting only the user table while leaving derived data and downstream copies.
- Copying production personal data into dev, demos, tests, or eval datasets.
- Relying on app-store privacy declarations instead of checking actual SDK behavior.

## Evidence Requirements

Privacy findings need:

- data subject and personal data type;
- collection, storage, processing, sharing, or deletion path;
- purpose/lawful-basis/notice/consent/retention evidence if visible;
- user or regulatory impact;
- code/config/vendor/log/index evidence;
- caveat that legal applicability may need counsel;
- false-positive checks for anonymization, aggregation, on-device-only processing, encryption boundaries, and external compliance evidence.

## Severity Guidance

- `critical`: large-scale or sensitive personal data exposure, unlawful irreversible processing, child/health/financial/biometric data misuse, or deletion/rights failure with severe regulatory/user harm.
- `high`: core workflow processes personal data without clear purpose, notice, consent where needed, retention control, or vendor governance.
- `medium`: important personal data flow has incomplete minimization, retention, rights, telemetry, or disclosure controls.
- `low`: localized privacy hygiene issue with limited user impact.
- `info`: documentation or evidence gap that needs privacy-owner confirmation.

## Confidence Guidance

- `high`: personal data flow and missing/incorrect control are directly evidenced in code/config/docs.
- `medium`: data flow is clear but external privacy records, contracts, or notices are unavailable.
- `low`: possible personal data path inferred from naming or SDK presence.

## False-Positive Guidance

- Data may be anonymized, aggregated, or processed only on-device; verify re-identification and transmission.
- Legal basis may be documented outside the repository.
- Retention/deletion may be enforced by warehouse, data platform, or cloud lifecycle policies outside code.
- Vendor contracts and transfer mechanisms are usually outside source control.
- Security controls such as encryption do not remove privacy obligations.

## Remediation Patterns

- Create or update data inventory and processing record for the feature.
- Remove unnecessary fields or reduce precision/duration.
- Make optional processing truly optional and withdrawal-capable.
- Add deletion/retention propagation to logs, analytics, warehouses, caches, indexes, backups, and AI artifacts.
- Redact or tokenize sensitive telemetry before it leaves the process.
- Add vendor/SDK review gates with purpose, data categories, region, retention, and subprocessor evidence.
- Configure AI provider data retention/training controls and avoid sending unnecessary PII.
- Add tests or operational checks for deletion and rights workflows.

## Good Finding Example

Title: Account deletion leaves personal chat embeddings in the vector store indefinitely

Evidence summary: `delete_account` removes `users` and `messages`, but embeddings are stored in `rag_vectors` by `document_id` without user ownership metadata or TTL. The deletion job does not query the vector store, and there is no retention policy for embeddings. These embeddings are derived from user-uploaded documents and can still be retrieved by document ID.

Severity: high if user uploads commonly contain personal or sensitive data.

Confidence: high when deletion code and vector schema are visible.

## Weak Or Unacceptable Finding Example

"This app may violate GDPR."

Reject this. Replace it with a specific personal data flow, missing control, and implementation evidence, and note that legal applicability requires counsel.

## Source Summary

This first-pass lens is grounded in GDPR article text, NIST Privacy Framework, NIST SP 800-122, NIST SP 800-53 Rev. 5, NISTIR 8062, EDPB consent guidance, ICO DPIA/data-sharing guidance, FTC personal information guidance, CCPA official information, OWASP privacy/logging guidance, Apple and Google mobile privacy docs, Android permissions, OpenAI data controls, NIST AI RMF, and minimization research.
