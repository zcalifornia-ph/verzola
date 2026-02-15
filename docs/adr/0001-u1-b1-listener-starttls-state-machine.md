# ADR 0001: U1-B1 Listener and STARTTLS State Machine

## Status

Accepted for Bolt U1-B1 implementation.

## Context

Unit U1 requires inbound SMTP ingress with STARTTLS capability, deterministic protocol behavior, and explicit failure mapping.

## Decision

- Implement a dedicated `ListenerConfig` model with validation.
- Implement a line-oriented SMTP command loop with explicit state:
  - `ehlo_seen`
  - `tls_active`
- Expose TLS handshake behind a `TlsUpgrader` trait so protocol logic is testable without hard-coding a TLS library in this bolt.
- Map temporary handshake failures to `454 4.7.0`.
- Require `EHLO` after successful STARTTLS before accepting envelope commands.

## Consequences

- Positive:
  - deterministic and testable SMTP/STARTTLS behavior,
  - clean seam for plugging in Rust TLS stack in a later bolt,
  - protocol compliance checks available through integration tests.
- Tradeoff:
  - this bolt does not yet ship production cryptographic handshake implementation; it defines the interface and protocol contract for that adapter.
