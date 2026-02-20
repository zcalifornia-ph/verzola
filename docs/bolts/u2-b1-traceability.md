# Bolt U2-B1 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: outbound session orchestration.
- Subtasks:
  - Design: outbound transaction lifecycle and MX selection strategy.
  - Implement: receive from Postfix and establish remote SMTP sessions.
  - Test: success path and transient failure handling.
  - Docs: outbound mode configuration examples.
  - Review: protocol behavior review with mail ops stakeholders.

## Context Summary

- U1 completed inbound listener, streaming relay, and inbound policy/telemetry.
- U2-B1 is the first outbound data-plane slice for Postfix relayhost mode.
- Outbound flow must preserve Postfix queue ownership via retry-safe temporary failures.
- MX selection strategy must be deterministic and test-covered before policy mapping expansion.
- Existing inbound suites must remain green while introducing outbound behavior.
- Repository has no external DNS/MX client yet; bolt requires a trait seam for resolver pluggability.
- Build artifacts and docs must keep lifecycle traceability aligned with `REQUIREMENTS.md`.

## File-Level Plan and Outputs

- Updated:
  - `verzola-proxy/src/lib.rs`
- Added:
  - `verzola-proxy/src/outbound/mod.rs`
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `docs/outbound-relay-configuration.md`
  - `docs/adr/0004-u2-b1-outbound-session-orchestration.md`
  - `docs/reviews/u2-b1-protocol-behavior-review.md`

## Acceptance Run

- Command:
  - `cargo test`
- Result:
  - passed (`11 integration tests total`, including outbound orchestration failover + temporary-failure coverage).
- Completed:
  - `2026-02-18`

## NFR/Risk Notes

- Reliability:
  - MX resolution/connect/bootstrap failures map to temporary `451` for retry safety.
- Performance:
  - outbound DATA path streams line-by-line with max-line guardrails.
- Security/operational safety:
  - explicit SMTP sequencing checks reduce undefined transaction states before policy-layer work.
