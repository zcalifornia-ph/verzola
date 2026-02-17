# Bolt U1-B2 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: stream inbound SMTP command/DATA relay to loopback Postfix.
- Subtasks:
  - Design: streaming pipeline and backpressure behavior.
  - Implement: command/data relay to loopback Postfix endpoint.
  - Test: large message and concurrent session tests.
  - Docs: integration notes for `main.cf` and `master.cf`.
  - Review: performance review against memory/latency targets.

## Context Summary

- U1-B1 already established listener + STARTTLS state machine and reply mapping.
- Inbound relay must preserve Postfix queue ownership semantics by using temporary failures on relay problems.
- Existing proxy loop is line-oriented and suitable for bounded streaming extension.
- Postfix loopback endpoint for inbound handoff is `localhost:2525`.
- Backpressure safety requires incremental DATA forwarding, not full-message buffering.
- U1-B2 acceptance requires both large-message coverage and concurrent-session coverage.
- Existing tests already include STARTTLS behavior and must remain passing.
- Docs/review artifacts are expected under `docs/` for traceability.

## File-Level Plan and Outputs

- Updated:
  - `verzola-proxy/src/inbound/mod.rs`
  - `verzola-proxy/src/main.rs`
  - `verzola-proxy/tests/inbound_starttls.rs`
- Added:
  - `verzola-proxy/tests/inbound_forwarder.rs`
  - `docs/adr/0002-u1-b2-streaming-forwarder.md`
  - `docs/inbound-postfix-integration.md`
  - `docs/reviews/u1-b2-performance-review.md`

## Acceptance Run

- Command:
  - `cargo test`
- Result:
  - passed (`5 passed; 0 failed`) across inbound STARTTLS + forwarding integration coverage.
- Completed:
  - `2026-02-16`

## NFR/Risk Notes

- Reliability:
  - relay failures map to temporary SMTP failure (`451`) for retry safety.
- Performance:
  - DATA is relayed line-by-line with bounded line length to avoid large buffering spikes.
- Interoperability:
  - relay session performs upstream `EHLO` handshake before envelope/data forwarding.
