# ADR 0006: U2-B3 Outbound TLS Policy Application

## Status

Accepted for Bolt U2-B3 implementation.

## Context

Unit U2 requires outbound policy-controlled transport behavior where VERZOLA can run in compatibility-first mode (`opportunistic`) or strict transport mode (`require-tls`) while preserving retry-safe Postfix semantics.

U2-B1 and U2-B2 established outbound orchestration and deterministic `250/4xx` status mapping, but they did not yet apply recipient-domain TLS policy during outbound relay session establishment.

## Decision

- Add explicit outbound TLS policy model:
  - `OutboundTlsPolicy::{Opportunistic, RequireTls}`.
- Add per-domain policy overrides:
  - `OutboundDomainTlsPolicy` for recipient-domain rules,
  - configuration validation rejects duplicate normalized domains.
- Evaluate effective policy in deterministic order:
  - first matching per-domain rule,
  - otherwise global outbound policy.
- Apply policy during remote MX bootstrap:
  - detect STARTTLS advertisement from EHLO capabilities,
  - attempt STARTTLS negotiation boundary when advertised.
- Enforce policy behavior:
  - `opportunistic`: if STARTTLS fails/unavailable, reopen plaintext session and continue.
  - `require-tls`: on STARTTLS absence/failure, return policy error mapped to `451 4.7.5` defer path.
- Extend session summary telemetry with policy and fallback counters to support observability/review.

## Consequences

- Positive:
  - outbound relay now has deterministic TLS policy behavior per recipient domain.
  - downgrade handling is explicit and test-covered.
  - strict policy failures preserve retry-safe semantics for Postfix queue ownership.
- Tradeoff:
  - STARTTLS negotiation is modeled as a policy decision boundary in this phase; full cryptographic adapter/wiring remains tracked in later bolts.
