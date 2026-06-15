# Security Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The security lens reviews exploitable weaknesses in application code, configuration, architecture, APIs, clients, mobile apps, model-integrated systems, and operational interfaces. It prioritizes attacker-controlled data, privilege boundaries, identity and session state, sensitive data, supply-chain inputs, unsafe defaults, and AI/RAG/agentic attack surfaces.

Security findings require an abuse scenario. A finding should say who the actor is, what they control, which trust boundary they cross, what asset is affected, and why existing controls are absent or insufficient.

## Subtopic Taxonomy

- Authentication: identity proofing, login flows, MFA/step-up, passwordless, account recovery.
- Authorization: object, function, relationship, tenant, policy enforcement, deny by default.
- Session and token lifecycle: issuance, storage, expiration, rotation, revocation, replay, OAuth/OIDC.
- Input validation and injection: SQL/NoSQL/LDAP/OS command/template/HTML/JS/CSS/URL/Markdown/parser/deserialization contexts.
- Secrets and credentials: source, artifacts, client bundles, logs, prompts, environment, rotation.
- Cryptography: key lifecycle, random, password hashing, signatures, encryption modes, TLS.
- Trust boundaries and secure defaults: config, debug, CORS, TLS, local bypasses, environment separation.
- API security: BOLA, BFLA, mass assignment, resource consumption, SSRF, unsafe API consumption.
- Browser/web security: XSS, CSRF, CSP, cookies, clickjacking, third-party JS, client/server contracts.
- Mobile security: local storage, backups, WebViews, platform IPC, deep links, network trust, release builds.
- Logging and telemetry exposure: logs, traces, metrics, analytics, prompts, retrieved documents, model outputs.
- Supply chain: vulnerable dependencies, build scripts, packages, model/data/provider dependencies. First-pass only; dedicated lens later.
- AI security: prompt injection, RAG poisoning, insecure output handling, excessive agency, tool manipulation, sensitive disclosure.

## High-Value Review Questions

- What can an unauthenticated user do?
- What can a low-privilege user do to another user, tenant, or organization?
- Which code turns untrusted input into a query, command, template, URL fetch, deserialization, or model instruction?
- Where are tokens and sessions created, stored, refreshed, invalidated, and logged?
- Are secrets ever committed, bundled, printed, sent to vendors, or exposed to users?
- What defaults are unsafe if production config is missing or wrong?
- Which controls run server-side versus client-side only?
- For AI systems, what can prompt injection influence, and what can tools do?

## Concrete Signals

- Resource loaded by request ID with no ownership/tenant policy check.
- Role check substitutes for object-level policy.
- User input concatenated into a command, query, template, URL, or prompt.
- OAuth flow missing current protections such as redirect validation, state/issuer checks, PKCE where applicable, or token replay mitigation.
- Secrets in repository, mobile package, frontend bundle, CI logs, prompt logs, or telemetry.
- Debug, permissive CORS, trust-all TLS, or disabled auth as production default.
- Request/response bodies and headers logged without redaction.
- LLM tools execute actions from model output without non-model policy enforcement.

## Anti-Patterns

- Client-side authorization.
- Denylist-only injection prevention.
- JWT-only revocation for high-risk sessions.
- Shared admin token for service or agent actions.
- Trusting RAG documents as instructions.
- Treating model guardrails as the only security boundary.
- Logging full payloads to make debugging easier.
- Mobile app enforcing business authorization locally instead of server-side.

## Evidence Requirements

Security findings must include:

- attacker or misuse actor;
- input or capability controlled by the actor;
- trust boundary crossed;
- vulnerable code/config path with line ranges where possible;
- affected asset;
- exploit or misuse scenario;
- impact;
- false-positive checks;
- remediation that addresses root cause.

For secrets, do not reproduce full secret values. Show a safe fingerprint or partial classification only.

## Severity Guidance

- `critical`: remote code execution, production credential compromise, cross-tenant sensitive data access, account takeover at scale, critical AI agent action without control, or destructive privileged mutation.
- `high`: sensitive data exposure, authz bypass, serious token/session flaw, exploitable injection, unsafe agent tool access, or high-impact mobile/API issue.
- `medium`: plausible exploit with constrained access, limited data, additional preconditions, or unclear reachability.
- `low`: defense-in-depth, hardening, low-sensitive data exposure, or unclear exploitability.
- `info`: security hygiene with no current exploit path.

## Confidence Guidance

- `high`: source-to-sink, actor-to-resource, or control-bypass path is directly evidenced.
- `medium`: issue is likely but depends on middleware, provider, framework, deployment, gateway, database policy, or build variant behavior.
- `low`: suspicious pattern without complete trace.
- `hypothesis`: preserve for notes unless verification steps are specific.

Confidence is not severity. A low-confidence critical hypothesis should not be reported as a critical finding until verified.

## False-Positive Guidance

Before filing, check:

- framework middleware and generated policies;
- API gateways, service meshes, WAFs, and identity provider settings;
- database row-level security and constraints;
- build variants and production config;
- generated or test-only code;
- redaction processors and logging pipeline config;
- external orchestrator approval gates for agents.

## Remediation Patterns

- Enforce server-side authorization close to resource access.
- Use parameterized APIs, safe encoders, structured parsers, and allow-list validation.
- Use current OAuth/session practices, rotate refresh tokens, and scope tokens narrowly.
- Move and rotate secrets; redact logs before export.
- Fail closed in production config.
- Use vetted crypto libraries and platform key stores.
- Add negative tests for authz, injection, SSRF, token lifecycle, mobile release config, and AI prompt/tool abuse.
- For AI, separate instructions from data, constrain tools, validate outputs, and enforce approvals outside the model.

## Good Finding Example

Title: Project detail endpoint allows cross-tenant reads by ID

Evidence summary: `GET /projects/{id}` loads the project by route ID and returns it after only checking that the requester is authenticated. The list endpoint scopes by tenant, but the detail endpoint does not compare project tenant to actor tenant. A user who learns another tenant's project ID can retrieve its metadata.

Severity: high if project data is sensitive; critical if cross-tenant secrets or regulated data are returned.

Confidence: high if no middleware/RLS policy is present on the path.

## Weak Or Unacceptable Finding Example

"The app might have injection because it uses strings."

Reject this. It lacks source, sink, input control, execution context, exploit scenario, and framework false-positive checks.

## Source Summary

The first-pass security lens is grounded in NIST SSDF/CSF, OWASP ASVS 5.0, OWASP Top 10, OWASP API Security 2023, OWASP WSTG, OWASP Authorization and Injection cheat sheets, IETF OAuth 2.0 Security BCP RFC 9700, CWE Top 25, OWASP MASVS/MASTG, OWASP LLM Top 10, OWASP prompt/RAG/agent/model-ops cheat sheets, and NIST AI 600-1.

