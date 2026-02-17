# ADR 0003: U1-B3 Inbound Policy Enforcement and Telemetry

## Status

Accepted for Bolt U1-B3 implementation.

## Context

Unit U1 requires deterministic inbound policy behavior for plaintext vs TLS sessions and auditable evidence of policy outcomes before outbound/control-plane work proceeds.

## Decision

- Add explicit inbound policy mode to listener configuration:
  - `opportunistic`
  - `require-tls`
- Enforce policy at envelope/data entry points (`MAIL`, `RCPT`, `DATA`) after SMTP ordering checks.
- In `require-tls`, reject plaintext envelope/data commands with `530 5.7.0 Must issue STARTTLS first`.
- Add configuration validation guard:
  - reject `require-tls` if `advertise_starttls` is disabled.
- Extend `SessionSummary` with a stable session telemetry schema:
  - policy mode,
  - STARTTLS attempts/failures,
  - require-tls rejection count,
  - relay temporary failure count.

## Consequences

- Positive:
  - deterministic policy behavior with explicit SMTP mappings,
  - session-level telemetry for acceptance and regression tests,
  - fail-fast config validation for unsafe policy combinations.
- Tradeoff:
  - telemetry is session-local in this bolt; aggregate metrics/log sinks remain planned for Unit U5.
