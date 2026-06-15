# Dependency Supply Chain Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The dependency-supply-chain lens reviews whether supplied code, artifacts, tools, containers, models, datasets, and licenses are selected, pinned, verified, inventoried, maintained, updated, and distributed safely. It covers third-party packages, lockfiles, package-manager semantics, transitive dependencies, CI/CD actions and scripts, release artifacts, SBOMs, provenance, signatures, containers, license policy, and AI/model/data artifacts.

Findings should not be raw scanner output or vague "dependency risk." They need a supplied artifact, trust boundary, evidence of weak control, and a plausible impact path.

## Subtopic Taxonomy

- Inventory: manifests, lockfiles, SBOMs, vendored code, generated code, container layers, binaries, models, datasets.
- Resolution: version ranges, lockfiles, git/path/URL dependencies, mutable tags, dependency groups, platform variants.
- Vulnerabilities: advisories, scanners, reachability, runtime exposure, remediation ownership, risk acceptance.
- Malicious packages: typosquatting, dependency confusion, maintainer/account takeover, install scripts, remote payloads.
- Provenance: SLSA, in-toto, build attestations, trusted publishing, source/build relationship.
- Signatures: Sigstore, registry signatures, artifact signing, transparency logs, identity policy.
- CI/CD supply chain: actions, orbs, remote shell scripts, build tools, runner images, tokens, secrets.
- Containers: base images, digests, tags, registry trust, OS packages, rebuild cadence, runtime downloads.
- Licensing: SPDX identifiers, license expressions, notices, distribution obligations, datasets and models.
- OSS health: maintainers, ownership, deprecation, unmaintained advisories, project governance.
- AI artifacts: model weights, adapters, prompts, RAG corpora, embeddings, eval datasets, unsafe serialization.

## High-Value Review Questions

- What supplied artifacts execute or ship with this product?
- Are deployable artifacts built from reproducible/pinned dependency resolution?
- What third-party code runs during install, build, test, release, and deploy?
- Are release artifacts bound to source, build workflow, signer identity, digest, and provenance?
- Are vulnerability scanner results triaged by runtime exposure and remediation path?
- Does the SBOM match the actual artifact, including containers, vendored code, binaries, and AI artifacts?
- Are CI tokens/secrets exposed to unpinned actions, remote scripts, or dependency lifecycle hooks?
- Are licenses and notices checked for the artifact as distributed?

## Concrete Signals

- Missing lockfile for deployable artifact.
- Floating `latest`, branch, tag, PR ref, or wildcard dependency in production build.
- Unpinned GitHub Action or container image in privileged workflow.
- CI pipes remote script into shell while secrets are available.
- Package install hooks run with cloud, registry, or signing credentials present.
- SBOM generated from source tree but release image includes untracked OS packages, binaries, or model files.
- Critical advisory is present in runtime dependency with no triage or update path.
- Package publishing uses long-lived tokens where trusted publishing is available.
- Model weights or datasets are downloaded by mutable name and loaded with unsafe serialization.
- License report excludes transitive, vendored, generated, model, dataset, or container dependencies.

## Anti-Patterns

- "We have Dependabot" as a substitute for triage and release gating.
- "Signed" but signer identity and provenance policy are not checked.
- Checksums stored beside mutable downloads on the same channel.
- SBOM produced once and not regenerated per release.
- Security scanning only application dependencies while CI tools and containers are ignored.
- Download count, popularity, or official-looking namespace treated as trust.
- License checks only direct source dependencies while shipping containers, mobile apps, desktop installers, fonts, datasets, or models.

## Evidence Requirements

Dependency-supply-chain findings need:

- artifact or dependency identity;
- source, version, digest, signer, lockfile, SBOM, or provenance evidence;
- trust boundary and execution/distribution context;
- control gap such as floating resolution, missing verification, stale inventory, or untriaged advisory;
- impact path such as code execution, credential exposure, vulnerable runtime, distribution violation, or incident-response blind spot;
- false-positive checks for external policy gates, private registries, admission controllers, hermetic builds, and legal approvals.

## Severity Guidance

- `critical`: supplied artifact can execute in a critical production path, exfiltrate credentials, sign/publish/deploy releases, corrupt safety/financial/identity data, or force major distribution halt.
- `high`: core build, deploy, runtime, or distributed artifact has a material unverified, vulnerable, or mutable supply-chain dependency.
- `medium`: important dependency or artifact lacks verification, inventory, triage, license, or maintenance control.
- `low`: localized low-impact metadata, dev-only, or documentation gap.
- `info`: supply-chain hygiene improvement without demonstrated risk path.

## Confidence Guidance

- `high`: manifest/build/release/CI evidence directly shows the supplied artifact and missing control.
- `medium`: repo evidence is strong but external controls may exist outside the inspected tree.
- `low`: signal is plausible but artifact criticality, runtime exposure, or policy state is unclear.

## False-Positive Guidance

- Enterprise registries, admission controllers, CI policies, or package mirrors may enforce pins, signatures, and SBOMs outside the repo.
- Libraries can legitimately omit lockfiles when downstream applications own reproducible resolution.
- Vulnerability scanners may over-report unreachable or patched packages.
- Mature stable dependencies can have low activity without being abandoned.
- Legal/commercial approvals may exist outside the repo.

## Remediation Patterns

- Pin dependency resolution and review lockfile diffs.
- Generate SBOMs from release artifacts and attach them to releases.
- Gate dependency changes with vulnerability, license, and package-health review.
- Use trusted publishing, provenance attestations, and signature verification with identity policy.
- Pin CI actions, images, and tools by immutable digest/commit.
- Run untrusted install/build scripts in secret-free sandboxes.
- Scan built container images and rebuild on base image vulnerabilities.
- Track high-risk dependencies for maintainer/ownership changes.
- Treat model weights, datasets, prompts, and embeddings as supplied artifacts with provenance, safe loading, and license review.

## Good Finding Example

Title: Release workflow executes unpinned third-party action with package publish token

Evidence summary: `.github/workflows/release.yml` uses `some/action@v1` in the npm publish job. The job grants `id-token: write`, exposes `NPM_TOKEN`, and runs before `npm publish`. The action ref is a mutable tag, so an upstream tag move or account compromise could execute arbitrary code with the publish token and release permissions.

Severity: high if the package is public or consumed by production systems.

Confidence: high when the workflow, mutable ref, and exposed token are visible.

## Weak Or Unacceptable Finding Example

"There are many dependencies."

Reject this. It lacks a specific artifact, trust boundary, weak control, and impact path.

## Source Summary

The first-pass dependency-supply-chain lens is grounded in SLSA 1.2, Sigstore, SPDX, CycloneDX, OWASP SCVS, NIST SSDF, OpenSSF Scorecard and Best Practices, OSV/OSV-Scanner, GitHub supply-chain and Actions docs, npm/PyPI/Cargo/RustSec/Go package-manager docs, TUF, in-toto, Docker, Kubernetes, SPDX License List, REUSE, OWASP Secure AI Model Ops, Hugging Face artifact security docs, Codecov incident details, and XZ/npm ecosystem research.

