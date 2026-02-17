# Bolt U1-B3 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: inbound policy enforcement and telemetry.
- Subtasks:
  - Design: policy decision points and telemetry schema.
  - Implement: `opportunistic` and `require-tls` handling for inbound sessions.
  - Test: policy matrix tests and telemetry assertion tests.
  - Docs: policy behavior reference for inbound paths.
  - Review: operational readiness review with SRE.

## Context Summary

- U1-B1 delivered listener + STARTTLS state machine and command sequencing guardrails.
- U1-B2 delivered streaming relay to loopback Postfix with temporary failure mapping.
- U1-B3 adds explicit policy decision points for plaintext-vs-TLS envelope handling.
- Policy behavior must be deterministic and testable before outbound/control-plane expansion.
- Existing integration suites for U1-B1/U1-B2 must remain passing.
- A stable telemetry schema is required for later operational aggregation work in Unit U5.
- `require-pq` remains intentionally out of scope for this bolt and is tracked under Unit U4.
- Current implementation favors small, local schema changes to avoid premature global metrics coupling.

## File-Level Plan and Outputs

- Updated:
  - `verzola-proxy/src/inbound/mod.rs`
  - `verzola-proxy/src/main.rs`
  - `verzola-proxy/tests/inbound_starttls.rs`
  - `verzola-proxy/tests/inbound_forwarder.rs`
- Added:
  - `verzola-proxy/tests/inbound_policy_telemetry.rs`
  - `docs/inbound-policy-telemetry.md`
  - `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`
  - `docs/reviews/u1-b3-operational-readiness.md`

## Acceptance Run

- Command:
  - `cargo test`
- Result:
  - passed (`9 integration tests + existing suites all green`).
- Completed:
  - `2026-02-17`

## NFR/Risk Notes

- Security:
  - deterministic `require-tls` enforcement with explicit `530` mapping.
- Reliability:
  - temporary TLS failures continue to map to `454`; relay failures map to `451`.
- Observability:
  - session telemetry schema added for STARTTLS and policy outcomes, ready for future aggregation.
