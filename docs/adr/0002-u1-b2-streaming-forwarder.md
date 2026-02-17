# ADR 0002: U1-B2 Streaming Forwarder to Postfix Loopback

## Status

Accepted for Bolt U1-B2 implementation.

## Context

Unit U1 requires inbound SMTP sessions to be forwarded to Postfix on loopback (`localhost:2525`) without full-message buffering, while preserving deterministic SMTP behavior and operational safety under concurrent traffic.

## Decision

- Extend `ListenerConfig` with optional `postfix_upstream_addr` to explicitly enable relay mode.
- Establish the Postfix relay connection lazily per client session and perform an upstream `EHLO` handshake once connected.
- Relay envelope and DATA flow to Postfix:
  - command relay for `MAIL`, `RCPT`, `DATA`, and `QUIT`,
  - DATA block relay in a line-by-line stream, bounded by `max_line_len`.
- Apply backpressure using blocking writes with flush per relayed DATA line to avoid unbounded application buffering.
- Map relay failures to temporary SMTP responses (`451`) so senders can retry safely.
- Add `serve_n` support in `InboundListener` to validate concurrent-session behavior in integration tests.

## Consequences

- Positive:
  - loopback Postfix integration is testable end-to-end in the inbound path,
  - large DATA payloads are forwarded without whole-message accumulation,
  - concurrent-session behavior is covered by integration tests.
- Tradeoff:
  - relay mode currently forwards a bounded SMTP subset and does not yet include full telemetry/policy hooks (covered by U1-B3).
