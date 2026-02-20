# Bolt U2-B3 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: outbound TLS policy application.
- Subtasks:
  - Design: outbound policy evaluation order and fallback logic.
  - Implement: `opportunistic`, `require-tls`, and per-domain rules.
  - Test: policy coverage tests for supported modes.
  - Docs: outbound policy examples including defer behavior.
  - Review: security review for downgrade resistance.

## Context Summary

- U2-B1 delivered outbound orchestration and deterministic MX failover.
- U2-B2 delivered deterministic Postfix-facing `250/4xx` contract behavior.
- U2-B3 needed policy-controlled outbound TLS behavior without breaking retry-safe semantics.
- Existing failed attempt already introduced policy types in `outbound/mod.rs` but left the build broken due incomplete test integration.
- Per-domain rule precedence was required so partner-specific policy could tighten or relax global behavior.
- Downgrade handling had to be explicit: fallback for opportunistic mode and defer for strict mode.
- Unit acceptance demanded preserving Postfix queue ownership on policy defer paths.

## File-Level Plan and Outputs

- Updated:
  - `verzola-proxy/src/outbound/mod.rs`
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `verzola-proxy/tests/outbound_status_contract.rs`
  - `docs/outbound-relay-configuration.md`
- Added:
  - `verzola-proxy/tests/outbound_tls_policy.rs`
  - `docs/adr/0006-u2-b3-outbound-tls-policy-application.md`
  - `docs/reviews/u2-b3-downgrade-resistance-review.md`
  - `docs/bolts/u2-b3-traceability.md`

## Acceptance Run

- Command:
  - `cargo test`
- Result:
  - passed (all suites green, including `outbound_tls_policy` with 6 policy-mode tests).
- Completed:
  - `2026-02-20`

## NFR/Risk Notes

- Security:
  - strict `require-tls` behavior defers with deterministic `451 4.7.5` when TLS requirements are unmet.
- Reliability:
  - policy defer paths remain retry-safe (`4xx`) for Postfix queue ownership.
- Traceability:
  - policy decision behavior, downgrade review notes, and acceptance evidence are now linked in dedicated U2-B3 artifacts.
