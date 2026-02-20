# Bolt U2-B2 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: delivery status contract normalization for outbound relay (`250/4xx`).
- Subtasks:
  - Design: mapping remote outcomes to Postfix-facing statuses.
  - Implement: deterministic status mapping and failure classification.
  - Test: contract tests for retry-safe semantics.
  - Docs: operator expectations and troubleshooting matrix.
  - Review: regression review against message safety requirements.

## Context Summary

- U2-B1 completed outbound session orchestration and deterministic MX candidate failover.
- Existing implementation relayed remote SMTP response lines directly for delivery stages.
- Unit U2 acceptance requires deterministic Postfix-facing behavior with `250` success and `4xx` retry/defer semantics.
- Reliability requirement `NFR-REL-02` mandates preserving Postfix queue ownership on transient/policy defer paths.
- Existing integration harness already models remote MX behavior and can be extended for status-contract assertions.
- Documentation needed an operator-facing status matrix to explain normalized outcomes and troubleshooting paths.

## File-Level Plan and Outputs

- Updated:
  - `verzola-proxy/src/outbound/mod.rs`
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `docs/outbound-relay-configuration.md`
- Added:
  - `verzola-proxy/tests/outbound_status_contract.rs`
  - `docs/adr/0005-u2-b2-delivery-status-contract.md`
  - `docs/reviews/u2-b2-message-safety-regression-review.md`

## Acceptance Run

- Command:
  - `cargo test`
- Result:
  - passed (`13` integration tests total across inbound + outbound suites, including `outbound_orchestration` and `outbound_status_contract`).
- Completed:
  - `2026-02-19`

## NFR/Risk Notes

- Reliability:
  - remote delivery-stage non-success outcomes map to retry-safe `451` responses.
- Message safety:
  - success `250` is emitted only for normalized delivery acceptance checkpoints.
- Traceability:
  - ADR, operator matrix, and regression review were added for contract-level auditability.
