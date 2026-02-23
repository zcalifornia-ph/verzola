# ADR 0010: U4-B1 Negotiated Group Classification

## Status

Accepted for Bolt U4-B1 implementation.

## Context

Unit U4 introduces TLS capability detection and `require-pq` policy behavior, but the proxy did not yet have a reusable parser/classifier for negotiated TLS group metadata.
Without a deterministic classification layer, later PQ policy enforcement (`require-pq`) could become inconsistent across TLS adapter metadata formats and log representations.

## Decision

- Add a new TLS utility module at `verzola-proxy/src/tls/mod.rs`.
- Define a minimal classification model for negotiated groups:
  - `pq`,
  - `classical`,
  - `none`.
- Parse negotiated-group metadata from conservative, semi-structured formats:
  - `key=value`,
  - `key: value`,
  - key + next-token forms (for common log styles),
  - raw single-token group names.
- Normalize candidate group names to lowercase alphanumeric form before matching so case and punctuation variations map consistently.
- Classify hybrid groups (classical + PQ/KEM markers) as `pq`.
- Classify unknown or unmapped values as `none` (fail-safe) instead of inferring `pq` or `classical`.

## Consequences

- Positive:
  - U4-B1 now provides a deterministic, test-covered contract for negotiated-group parsing/classification.
  - Later `require-pq` policy decisions can reuse the same parser/classifier and inherit conservative defaults.
  - Metadata format variance across TLS adapters/logs is reduced through normalization.
- Tradeoff:
  - Numeric IDs and future group names without explicit markers are currently classified as `none` until mapping support is expanded in later bolts.
