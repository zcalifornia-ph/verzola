# Version v0.1.8 Documentation

## Title
Unit U2-B2 Delivery Status Contract Implementation (`250/4xx`) and Release Sync

## Quick Diagnostic Read

This version closes Unit U2 Bolt U2-B2 by making outbound Postfix-facing delivery outcomes deterministic and retry-safe.

Primary outcomes:

- outbound relay now enforces explicit stage-based status normalization,
- new contract tests prove retry behavior for transient and permanent upstream outcomes,
- requirements, ADR/review/traceability docs, README, and changelog are synchronized to this milestone.

## One-Sentence Objective

Ship Unit U2 Bolt U2-B2 with deterministic Postfix-facing delivery semantics (`250` on confirmed acceptance, `451` defer for retry-required outcomes) and traceable evidence artifacts.

## Scope of This Version

This version includes code, tests, and documentation updates:

- outbound status-contract implementation in `verzola-proxy`,
- outbound contract-focused integration testing,
- operator and lifecycle documentation for U2-B2,
- root version/changelog synchronization.

## Detailed Changes

## 1) Outbound Delivery Status Contract (U2-B2)

Updated:

- `verzola-proxy/src/outbound/mod.rs`

Implemented behavior:

- introduced explicit delivery-stage mapping for:
  - `RCPT` relay,
  - `DATA` command relay,
  - final DATA payload completion.
- normalized Postfix-facing success outcomes:
  - `250 2.1.5 Recipient accepted for remote delivery` on accepted recipient stage,
  - `250 2.0.0 Message accepted by remote MX` only after final remote acceptance.
- normalized non-success remote outcomes (`4xx`, `5xx`, or unexpected classes) to deterministic retry-safe defer:
  - `451 4.4.0 Delivery deferred for retry (stage=..., class=..., upstream=...)`.
- preserved staged transaction state reset only after final success, maintaining queue-safety semantics.

## 2) Contract Tests and Regression Guarding

Updated:

- `verzola-proxy/tests/outbound_orchestration.rs`
  - expected normalized success response messages now assert U2-B2 contract outputs.

Added:

- `verzola-proxy/tests/outbound_status_contract.rs`
  - `maps_remote_transient_rcpt_status_to_retry_safe_defer`
  - `maps_remote_permanent_data_status_to_retry_safe_defer`

Coverage intent:

- prove transient and permanent upstream responses both defer safely to Postfix,
- avoid queue-ownership ambiguity by enforcing `4xx` retry behavior.

## 3) Requirements and Lifecycle Documentation

Updated:

- `REQUIREMENTS.md`
  - marked `Bolt U2-B2` and all subtasks complete,
  - recorded completion date and acceptance evidence.
- `docs/outbound-relay-configuration.md`
  - added status mapping matrix and troubleshooting matrix.

Added:

- `docs/adr/0005-u2-b2-delivery-status-contract.md`
- `docs/reviews/u2-b2-message-safety-regression-review.md`
- `docs/bolts/u2-b2-traceability.md`

## 4) Root Documentation and Release Metadata

Updated:

- `README.md`
  - version marker advanced to `v0.1.8`,
  - repository snapshot includes `docs/version-v0.1.8-docs.md`.
- `CHANGELOG.md`
  - added `v0.1.8` release notes and manual cleanup targets.

Added:

- `docs/version-v0.1.8-docs.md` (this file).

## Traceability Links

- Requirements:
  - `REQUIREMENTS.md`
- Outbound implementation:
  - `verzola-proxy/src/outbound/mod.rs`
- Outbound tests:
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `verzola-proxy/tests/outbound_status_contract.rs`
- U2-B2 documentation:
  - `docs/outbound-relay-configuration.md`
  - `docs/adr/0005-u2-b2-delivery-status-contract.md`
  - `docs/reviews/u2-b2-message-safety-regression-review.md`
  - `docs/bolts/u2-b2-traceability.md`

## Validation Notes

Acceptance validation command:

- `cargo test` (run in `verzola-proxy`)

Observed results for implemented scope:

- inbound suites: `2 + 4 + 3` tests passed,
- outbound orchestration suite: `2` tests passed,
- outbound status-contract suite: `2` tests passed.

Validation run date:

- `2026-02-19`
