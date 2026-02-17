# Inbound Listener Setup (Unit U1 / Bolt U1-B1)

## Scope

This document covers the SMTP listener and STARTTLS negotiation slice implemented in `verzola-proxy/src/inbound/mod.rs`.

## Listener Configuration Model

`ListenerConfig` defines:

- `bind_addr`: TCP socket address for SMTP ingress.
- `banner_host`: hostname advertised in the `220` banner and `EHLO` replies.
- `advertise_starttls`: enables/disables `STARTTLS` capability advertisement.
- `inbound_tls_policy`: inbound envelope policy (`opportunistic` or `require-tls`).
- `max_line_len`: guardrail for command and DATA line length.

Validation rules:

- `banner_host` must be non-empty.
- `max_line_len` must be at least `512`.
- `require-tls` policy requires `advertise_starttls = true`.

## STARTTLS State Machine

Session flow:

1. Server sends `220 <host> ESMTP VERZOLA`.
2. Client sends `EHLO`/`HELO`.
3. Server advertises `STARTTLS` when enabled and not already active.
4. Client sends `STARTTLS`.
5. Server sends `220 Ready to start TLS`.
6. TLS upgrader runs:
   - success: session marks TLS active and requires new `EHLO`.
   - temporary failure: server maps to `454 4.7.0 ...`.

Protocol guardrails:

- `STARTTLS` before `EHLO` returns `503`.
- `STARTTLS` while TLS is already active returns `503`.
- `MAIL/RCPT/DATA` without required `EHLO` returns `503`.

Policy-specific envelope guardrails are documented in `docs/inbound-policy-telemetry.md`.

## Certificate Requirements (Production Adapter)

`NoopTlsUpgrader` is only for test and scaffolding. Production deployments must provide a real `TlsUpgrader` implementation that:

- loads X.509 certificate chain and private key from secured paths or secret mounts,
- supports TLS 1.2+ and server-preferred cipher configuration,
- optionally enforces client-certificate behavior if required by policy,
- surfaces handshake failures as temporary errors so SMTP can return `454`.

Recommended operational minimums:

- certificate/key files readable only by the VERZOLA service account,
- explicit certificate rotation process with reload behavior,
- startup preflight that fails fast on certificate parse/load errors.

## Validation Commands

When Rust toolchain is available:

```powershell
cd verzola-proxy
cargo test
```
