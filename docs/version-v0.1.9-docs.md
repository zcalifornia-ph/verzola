# Version v0.1.9 Documentation

## Title
Unit U2-B3 Outbound TLS Policy Application Completion and Unit U2 Closure

## Quick Diagnostic Read

This version closes Unit U2 by completing Bolt U2-B3 and finalizing outbound TLS policy behavior for relay sessions.

Primary outcomes:

- outbound relay now applies deterministic TLS policy (`opportunistic` / `require-tls`) with per-domain overrides,
- strict policy downgrade paths return retry-safe defers (`451 4.7.5`) instead of ambiguous failures,
- U2 documentation, traceability artifacts, README, and changelog are synchronized to the completed unit.

## One-Sentence Objective

Ship Unit U2 Bolt U2-B3 with policy-driven outbound TLS behavior, full integration coverage, and traceable lifecycle artifacts, then mark Unit U2 complete in `REQUIREMENTS.md`.

## Scope of This Version

This version includes code, tests, and documentation updates:

- outbound TLS policy logic in `verzola-proxy`,
- outbound policy-focused integration tests,
- U2-B3 ADR/review/traceability documentation,
- root version/changelog synchronization.

## Detailed Changes

## 1) Outbound TLS Policy Model and Session Behavior (U2-B3)

Updated:

- `verzola-proxy/src/outbound/mod.rs`

Implemented behavior:

- added explicit outbound policy enum:
  - `OutboundTlsPolicy::Opportunistic`
  - `OutboundTlsPolicy::RequireTls`
- added per-domain override model:
  - `OutboundDomainTlsPolicy`,
  - config validation for duplicate normalized domains.
- introduced deterministic policy resolution:
  - per-domain override first,
  - fallback to global policy.
- added STARTTLS capability detection from remote EHLO reply.
- implemented opportunistic fallback:
  - if STARTTLS is unavailable or negotiation fails, reopen plaintext session and continue.
- implemented strict defer behavior:
  - when `require-tls` is effective and TLS cannot be established, return policy error mapped to `451 4.7.5`.
- expanded outbound session summary for traceability:
  - effective policy,
  - TLS negotiated flag,
  - opportunistic fallback counter,
  - policy defer counter.

## 2) Outbound Policy Integration Tests

Updated:

- `verzola-proxy/tests/outbound_orchestration.rs`
- `verzola-proxy/tests/outbound_status_contract.rs`

Change:

- config constructors now inherit new policy fields via `..OutboundListenerConfig::default()` to keep existing suites compile-safe.

Added:

- `verzola-proxy/tests/outbound_tls_policy.rs`

Coverage includes:

- opportunistic success when STARTTLS is not advertised,
- opportunistic fallback when STARTTLS negotiation fails,
- strict `require-tls` defer behavior,
- per-domain stricter override precedence,
- per-domain relaxing override precedence,
- duplicate-domain policy validation failure path.

## 3) Requirements and Traceability

Updated:

- `REQUIREMENTS.md`
  - marked `Bolt U2-B3` and all subtasks complete,
  - recorded completion date (`2026-02-20`) and acceptance evidence,
  - marked Unit U2 acceptance criteria and deliverables complete.

Added:

- `docs/adr/0006-u2-b3-outbound-tls-policy-application.md`
- `docs/reviews/u2-b3-downgrade-resistance-review.md`
- `docs/bolts/u2-b3-traceability.md`

## 4) Root Documentation and Release Metadata

Updated:

- `README.md`
  - version marker advanced to `v0.1.9`,
  - repository snapshot includes U2-B3 artifacts and new suite,
  - quick-start commands include `outbound_tls_policy`,
  - roadmap/progress/validation notes now reflect completed Unit U2.
- `CHANGELOG.md`
  - added `v0.1.9` release notes and manual cleanup targets.

Added:

- `docs/version-v0.1.9-docs.md` (this file).

## Traceability Links

- Requirements:
  - `REQUIREMENTS.md`
- Outbound implementation:
  - `verzola-proxy/src/outbound/mod.rs`
- Outbound tests:
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `verzola-proxy/tests/outbound_status_contract.rs`
  - `verzola-proxy/tests/outbound_tls_policy.rs`
- U2-B3 documentation:
  - `docs/outbound-relay-configuration.md`
  - `docs/adr/0006-u2-b3-outbound-tls-policy-application.md`
  - `docs/reviews/u2-b3-downgrade-resistance-review.md`
  - `docs/bolts/u2-b3-traceability.md`

## Validation Notes

Acceptance validation command:

- `cargo test` (run in `verzola-proxy`)

Observed results for implemented scope:

- inbound suites: `2 + 4 + 3` tests passed,
- outbound suites: `2 + 2 + 6` tests passed.

Validation run date:

- `2026-02-20`
