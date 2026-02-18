# ADR 0004: U2-B1 Outbound Session Orchestration

## Status

Accepted for Bolt U2-B1 implementation.

## Context

Unit U2 requires VERZOLA to accept Postfix relayhost sessions on loopback and orchestrate immediate outbound SMTP delivery attempts to remote MX hosts while preserving retry-safe behavior on transient failures.

## Decision

- Add a dedicated outbound relay module (`verzola-proxy/src/outbound/mod.rs`) with:
  - `OutboundListenerConfig` validation and bind contract,
  - `OutboundListener` session serving (`serve_one`, `serve_n`).
- Introduce an `MxResolver` trait and `MxCandidate` model to keep MX lookup strategy testable and replaceable.
- Determine outbound route on first valid `RCPT TO`:
  - extract recipient domain,
  - resolve MX candidates,
  - sort by `(preference, exchange)`,
  - attempt candidates sequentially until one session bootstraps successfully.
- Bootstrap remote MX session with SMTP preflight before relaying recipients:
  - read banner (`2xx` required),
  - send `EHLO` (`2xx` required),
  - send staged `MAIL FROM` (`2xx` required).
- Relay `RCPT`, `DATA`, and payload streaming to the selected remote MX session.
- Map resolver/connection/bootstrap errors to temporary response `451 4.4.0` so Postfix retains retry control.

## Consequences

- Positive:
  - outbound orchestration behavior is now implemented and integration-testable,
  - MX candidate failover is deterministic and explicit,
  - temporary failure mapping preserves safe retry semantics in Postfix.
- Tradeoff:
  - this bolt intentionally limits a transaction to one recipient domain and defers advanced outcome classification (`250/4xx` contract matrix) to U2-B2.
