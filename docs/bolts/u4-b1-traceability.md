# Bolt U4-B1 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: classify negotiated TLS group results as `pq`, `classical`, or `none`.
- Subtasks:
  - Design: result classification model (`pq`, `classical`, `none`).
  - Implement: parser for negotiated group metadata.
  - Test: classification tests across handshake fixtures.
  - Docs: classification mapping reference.
  - Review: crypto review for correctness.

## Context Summary

- Unit U2 added outbound STARTTLS policy behavior but left negotiated-group telemetry/classification out of scope for Unit U4.
- Unit U4 acceptance will require per-session negotiated group and PQ/classical classification capture.
- The `verzola-proxy` crate had no TLS utility module, so U4-B1 needed a new reusable parsing/classification surface.
- TLS adapter metadata formats vary (`key=value`, `key: value`, log tokens), so normalization and conservative parsing are required.
- Later `require-pq` policy enforcement must not infer PQ support from ambiguous metadata.
- Existing outbound and inbound integration suites act as regression guards while introducing the new TLS module.
- Risk register already flags hybrid/PQ interoperability instability, making deterministic classification a prerequisite.

## File-Level Plan and Outputs

- Added:
  - `verzola-proxy/src/tls/mod.rs`
  - `verzola-proxy/tests/tls_negotiation_classification.rs`
  - `docs/tls-negotiation-classification-reference.md`
  - `docs/adr/0010-u4-b1-negotiated-group-classification.md`
  - `docs/reviews/u4-b1-crypto-classification-review.md`
  - `docs/bolts/u4-b1-traceability.md`
- Updated:
  - `verzola-proxy/src/lib.rs`
  - `REQUIREMENTS.md`

## Acceptance Run

- Command:
  - `cargo test`
- Run location:
  - `verzola-proxy`
- Result:
  - passed (all suites green, including new `tls_negotiation_classification` with `3` passing tests).
- Completed:
  - `2026-02-23`

## NFR/Risk Notes

- Security:
  - conservative `none` fallback prevents false-positive PQ classification from ambiguous metadata.
- Reliability:
  - parsing is tolerant to common log metadata formatting variations without depending on a specific TLS adapter format.
- Compliance/Traceability:
  - ADR, mapping reference, review, and this traceability file record the classification contract for later U4 bolts.
