# API Contracts Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The API contracts lens reviews whether clients and providers can safely evolve independently. It covers REST/HTTP, GraphQL, gRPC/protobuf, event APIs, webhooks, generated clients, SDKs, schemas, errors, pagination, idempotency, deprecation, versioning, and consumer-provider verification.

API-contract findings should identify a boundary and a consumer/provider expectation. They should not be generic API style advice.

## Subtopic Taxonomy

- Contract artifacts: OpenAPI, JSON Schema, GraphQL schema, protobuf, AsyncAPI, SDK types, event schemas.
- Protocol semantics: HTTP methods/status/content negotiation, gRPC status codes, GraphQL response and error behavior.
- Compatibility: additive changes, field removal/rename/type changes, enum expansion, protobuf field reservations, schema versions.
- Versioning and lifecycle: route/header/date versions, SDK pins, webhook versions, deprecation, sunset, migration.
- Errors: machine-readable error type/code, status mapping, field errors, retry/remediation semantics.
- Lists and queries: pagination, cursors, ordering, filtering, links, limits.
- Mutations and webhooks: idempotency, duplicate intent, retryability, request/event IDs, unknown outcomes.
- Event contracts: message meaning, schema, delivery, ordering, deduplication, dead-letter behavior.
- Verification: contract tests, consumer-driven contracts, provider verification, breaking-change checks, generated-client CI.
- Auth semantics: security schemes, scopes, actor, tenant, resource ownership, webhook signatures.

## High-Value Review Questions

- What is the authoritative contract artifact for this boundary?
- Which clients or consumers depend on it, and can they deploy independently?
- Would an old client parse and correctly act on the new response/event/RPC?
- Are request, response, error, and event schemas validated in CI or only documented?
- Are list ordering, pagination, limits, and filtering explicit?
- Can clients safely retry mutations and webhooks after unknown outcomes?
- Do generated clients, SDK versions, webhook versions, and runtime API versions match?
- Are auth scopes, tenant context, and actor semantics accurately represented in the contract?

## Concrete Signals

- Contract artifact and implementation disagree about fields, types, enums, status codes, or examples.
- API change removes, renames, narrows, or makes required a field used by clients.
- Protobuf field number is reused or enum evolution is not considered.
- Endpoint returns generic errors that force clients to parse messages.
- Offset pagination uses unstable ordering or no maximum page size.
- POST mutation can be retried without durable idempotency semantics.
- Event payload lacks schema/version/event ID/order/deduplication contract.
- Generated client is stale or regenerated without CI diff checks.
- Security scheme omits required scopes, tenant, actor, or webhook signature behavior.

## Anti-Patterns

- Docs as the only contract.
- "Internal API" used as a reason to ignore compatibility between repos or deploy units.
- Hand-written mocks that are not provider-verified.
- Treating wire compatibility as business compatibility.
- Client parsing of localized/human-readable error strings.
- Cursor or idempotency key not bound to tenant, actor, filters, or request parameters.
- Generated code edited manually.

## Evidence Requirements

API-contract findings need:

- boundary type and producer/consumer relationship;
- authoritative contract artifact or proof that none exists;
- concrete mismatch, ambiguity, incompatible change, or missing verification;
- consumer-visible effect;
- deploy/versioning context;
- false-positive checks for gateways, generated serializers, schema registries, compatibility layers, and external CI.

## Severity Guidance

- `critical`: contract issue can break critical external integrations, corrupt data, double-execute irreversible actions, expose cross-tenant behavior, or stop revenue/safety/compliance workflows.
- `high`: core client/provider, SDK, webhook, or event workflow can break under normal deployment or retry.
- `medium`: important integration has material ambiguity, drift, or missing verification.
- `low`: localized or low-impact contract/documentation inconsistency.
- `info`: style or consistency improvement without demonstrated consumer risk.

## Confidence Guidance

- `high`: contract artifact, implementation, and consumer failure mode are directly evidenced.
- `medium`: boundary mismatch is visible but consumer set, gateway behavior, or external CI is uncertain.
- `low`: likely contract smell with unclear authoritative source or deploy independence.

## False-Positive Guidance

- API gateways may enforce schemas, auth scopes, pagination limits, or compatibility externally.
- Generated serializers may transform server models before the response leaves the service.
- Monorepos or release trains may deploy consumers and providers atomically.
- Schema registries or brokers may enforce event compatibility outside this repository.
- Public docs may be generated after build, not committed.

## Remediation Patterns

- Establish one authoritative schema and generate/validate from it.
- Add breaking-change checks for OpenAPI, GraphQL, protobuf, and AsyncAPI artifacts.
- Use additive evolution, deprecation windows, sunset/migration communication, and version pinning where needed.
- Stabilize machine-readable errors and retry/remediation semantics.
- Define list ordering, maximum limits, cursor binding, and filter validation.
- Add durable idempotency/request/event IDs for mutation and webhook retries.
- Verify consumer contracts in CI and provider deploy gates.
- Pin generated clients, SDKs, and webhook endpoints to compatible API versions.

## Good Finding Example

Title: Webhook contract changed `invoice.total` from integer cents to decimal string without versioning

Evidence summary: The webhook schema in `webhooks/openapi.yaml` documents `invoice.total` as integer cents. The new producer serializes a decimal string from the billing model, while the existing consumer generated type still expects integer. The webhook topic and version header are unchanged, and provider verification does not run for consumer fixtures.

Severity: high if external billing consumers depend on the webhook for reconciliation.

Confidence: high when schema, producer serialization, and consumer generated type are all visible.

## Weak Or Unacceptable Finding Example

"The API should follow REST best practices."

Reject this. It lacks boundary, contract artifact, consumer expectation, evidence, and impact.

## Source Summary

The first-pass API contracts lens is grounded in OpenAPI, JSON Schema, HTTP Semantics, Problem Details, Web Linking, Sunset header, GraphQL, gRPC, protobuf, AsyncAPI, Martin Fowler contract testing, Pact, Google Cloud API Design Guide, Zalando/Microsoft API guidelines, Stripe versioning, and AWS idempotent API guidance.

