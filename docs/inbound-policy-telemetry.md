# Inbound Policy Enforcement and Telemetry (Unit U1 / Bolt U1-B3)

## Scope

This document defines inbound SMTP policy behavior and session telemetry emitted by `verzola-proxy/src/inbound/mod.rs`.

## Policy Modes

`ListenerConfig` now includes `inbound_tls_policy`:

- `opportunistic` (`InboundTlsPolicy::Opportunistic`)
  - STARTTLS is offered (when configured), but plaintext `MAIL/RCPT/DATA` is still accepted after `EHLO`.
- `require-tls` (`InboundTlsPolicy::RequireTls`)
  - SMTP envelope/data commands are rejected until TLS is active.
  - Rejection mapping: `530 5.7.0 Must issue STARTTLS first`.

Validation guardrails:

- `inbound_tls_policy=require-tls` requires `advertise_starttls=true`.
- invalid combinations fail at listener bind time (`InvalidInput`), preventing unsafe startup.

## Decision Points

Inbound command handling decision points:

1. `EHLO/HELO`
   - enables envelope/data progression.
   - advertises `STARTTLS` only when configured and TLS is not already active.
2. `STARTTLS`
   - maps to `220 Ready to start TLS`, then runs `TlsUpgrader`.
   - temporary upgrade failures map to `454 4.7.0 ...`.
3. `MAIL/RCPT/DATA`
   - require `EHLO` ordering first (`503` when missing).
   - apply policy check:
     - `opportunistic`: allow plaintext path.
     - `require-tls`: reject plaintext with `530` until successful STARTTLS.

## Session Telemetry Schema

`SessionSummary` includes policy + telemetry fields:

- `inbound_tls_policy`: effective policy mode for the session.
- `telemetry.starttls_offered`: whether STARTTLS was configured for the listener.
- `telemetry.starttls_attempts`: number of STARTTLS commands received.
- `telemetry.tls_upgrade_failures`: temporary TLS upgrade failures (`454` path).
- `telemetry.require_tls_rejections`: policy rejections (`530 Must issue STARTTLS first`).
- `telemetry.relay_temporary_failures`: relay unavailability failures mapped to `451`.

These fields are intentionally session-scoped for deterministic test assertions and a stable schema before global metrics/log exporters are introduced in Unit U5.

## Validation Commands

```powershell
cd verzola-proxy
cargo test
```

Targeted policy and telemetry suite:

```powershell
cd verzola-proxy
cargo test --test inbound_policy_telemetry
```
