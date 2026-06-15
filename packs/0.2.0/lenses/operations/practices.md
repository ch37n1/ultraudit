# Operations Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The operations lens reviews whether the system can be configured, deployed, rolled back, operated, recovered, and improved safely. It covers runtime config, secrets, CI/CD, release gates, rollout and rollback, migration order, environment drift, runbooks, incident response, postmortems, and AI/model/provider operations.

Operations findings need an operational scenario: deploy, rollback, outage, incident, config error, credential rotation, migration failure, provider outage, or repeated incident.

## Subtopic Taxonomy

- Configuration: env separation, config schemas, safe defaults, startup validation.
- Secrets: injection, scope, rotation, artifact/log exposure, CI/CD secrets.
- Release engineering: repeatable builds, release gates, artifact integrity, deploy ownership.
- Rollout and rollback: progressive deploy, health checks, rollback of data/config/index/model changes.
- Migrations: deploy ordering, transactional/non-transactional behavior, expand-contract, failed migration repair.
- CI/CD: triggers, approvals, permissions, pinned actions/plugins, required checks, production environments.
- Environment drift: IaC, config drift, staging/prod parity, manual changes.
- Runbooks: alert ownership, diagnosis, mitigation, rollback, escalation, verification.
- Incidents and learning: emergency response, incident roles, postmortems, corrective actions.
- AI/model operations: model/prompt/index/tool rollout, provider quotas, fallback, eval gates, incident procedures.

## High-Value Review Questions

- What happens if required config is missing in production?
- Can secrets be rotated without code changes or broad redeploys?
- Who can deploy to production, and what gates can they bypass?
- Can a bad deploy be stopped or rolled back safely?
- Does rollback include migrations, jobs, indexes, models, and config?
- Can staging/prod drift be detected?
- Does every paging alert have an owner and runbook?
- Do incidents create tracked, verified corrective actions?
- Are AI model/prompt/index/tool changes operationally controlled?

## Concrete Signals

- Unvalidated environment variables change production behavior.
- Secrets are embedded in images, logs, CI commands, or client bundles.
- Deploy workflow can run from untrusted branches or skip tests/evals.
- Deployment marks process start as success while readiness is unsafe.
- Migration and code deploy require perfect atomic timing.
- Rollback plan ignores schema or vector index changes.
- Runbook is missing, stale, or says only "restart service."
- Terraform/IaC drift is not checked.
- Prompt/model changes are edited in production without release record.

## Anti-Patterns

- Production config copied from examples.
- Kubernetes Secret treated as complete secret management.
- Manual deploy steps remembered by one person.
- Rollback equals previous image only.
- Staging success treated as production proof despite known drift.
- Alert without owner or runbook.
- Postmortem action item "be more careful."
- AI provider dashboard as the whole operations plan.

## Evidence Requirements

Operations findings need:

- operational workflow or failure scenario;
- config/deploy/CI/runbook/IaC/migration/model artifact path;
- missing or unsafe control;
- expected impact during deploy, incident, rollback, or recovery;
- false-positive checks for external platform settings, runbooks, release systems, and separate IaC repos.

## Severity Guidance

- `critical`: operational gap can deploy malicious/untested code, expose production credentials, cause broad outage, irreversible data loss, or unsafe AI action.
- `high`: core system cannot be safely deployed, rolled back, configured, or operated.
- `medium`: important operational control is fragile, manual, or incomplete.
- `low`: low-risk operational hygiene gap.
- `info`: improvement opportunity without current production risk.

## Confidence Guidance

- `high`: repository artifacts directly show unsafe operational path.
- `medium`: platform UI, external CI/CD, IaC repo, or incident system may supply controls.
- `low`: production operations are mostly outside visible artifacts.

## False-Positive Guidance

- CI/CD protections may be configured in platform settings outside YAML.
- IaC, runbooks, and incident tickets may live in separate repositories/systems.
- Small local tools may not require full operational machinery.
- Maintenance-window deployments may intentionally trade availability for simplicity.

## Remediation Patterns

- Validate typed config at startup and fail closed.
- Use secret managers/platform secret injection with rotation and least privilege.
- Protect CI/CD deploy paths, pin third-party actions, and require gates.
- Use readiness gates, progressive rollout, deploy smoke tests, and rollback plans.
- Coordinate migrations with expand-contract and resumable operations.
- Detect drift and document intentional environment differences.
- Link alerts to owned, tested runbooks.
- Track postmortem actions to verified closure.
- Treat prompts, models, indexes, and AI tools as versioned deployable artifacts.

## Good Finding Example

Title: Rollback plan cannot restore previous RAG behavior after index rebuild

Evidence summary: The deploy workflow rebuilds the vector index in place and then deploys new prompt code. The rollback job redeploys the previous container image only; it does not restore the prior index or prompt/index compatibility record. A bad index build can continue serving stale or wrong chunks after rollback.

Severity: high if the RAG feature is core decision support.

Confidence: high if no external index snapshot/rollback is configured.

## Weak Or Unacceptable Finding Example

"Use Kubernetes best practices."

Reject this. It does not identify the operational workflow, unsafe artifact, failure scenario, or impact.

## Source Summary

The first-pass operations lens is grounded in AWS Operational Excellence, Google SRE release/launch/incident/postmortem material, Twelve-Factor config, Kubernetes ConfigMaps/Secrets/Deployments/probes, OWASP CI/CD and GitHub Actions secure-use guidance, Terraform drift, evolutionary database design/Flyway migration behavior, and OWASP Secure AI Model Ops.

