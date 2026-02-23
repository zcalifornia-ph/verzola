# Version v0.1.15 Documentation

## Title
Unit U4-B1 Negotiated Group Classification

## Quick Diagnostic Read

This version completes `Unit U4 / Bolt U4-B1` by adding a conservative negotiated-group classification layer for TLS capability detection.

Primary outcomes:

- `verzola-proxy` now has a reusable TLS metadata parser/classifier (`pq`, `classical`, `none`),
- handshake-fixture coverage validates parsing and classification edge cases,
- Unit U4-B1 is marked complete in `REQUIREMENTS.md` with dated evidence.

## One-Sentence Objective

Provide a deterministic, fail-safe negotiated-group classification contract that later `require-pq` policy handling can reuse without ambiguous PQ detection.

## Scope of This Version

This version includes:

- a new TLS classification module in `verzola-proxy`,
- a new integration test suite for classification fixtures,
- Unit U4-B1 documentation/review/ADR/traceability artifacts,
- root-document synchronization for U4-B1 completion (`README.md`, `CHANGELOG.md`).

## Detailed Changes

## 1) Negotiated Group Classification Implementation

Added:

- `verzola-proxy/src/tls/mod.rs`

Updated:

- `verzola-proxy/src/lib.rs`

Key implementation outcomes:

- classification model with `pq`, `classical`, and `none`,
- parsing for common metadata shapes (`key=value`, `key: value`, spaced token styles),
- normalization before mapping to reduce case/punctuation variance,
- conservative fallback to `none` for absent/placeholder/unknown group values.

## 2) Classification Validation Coverage

Added:

- `verzola-proxy/tests/tls_negotiation_classification.rs`

Coverage includes:

- classical named groups (`x25519`, `secp256r1`, `ffdhe3072`),
- hybrid/PQ-style names (`X25519MLKEM768`, `x25519_kyber768draft00`),
- parser robustness for spaced separators and mixed metadata tokens,
- conservative handling for unknown placeholders and unmapped numeric IDs.

## 3) U4-B1 Documentation and Review Artifacts

Added:

- `docs/tls-negotiation-classification-reference.md`
- `docs/adr/0010-u4-b1-negotiated-group-classification.md`
- `docs/reviews/u4-b1-crypto-classification-review.md`
- `docs/bolts/u4-b1-traceability.md`

Purpose:

- define the current mapping contract and limitations,
- record the design decision and conservative classification defaults,
- preserve bolt-level traceability and review evidence for later Unit U4 work.

## 4) Root Documentation Sync (`v0.1.15`)

Updated:

- `README.md`
- `CHANGELOG.md`

Sync points:

- `README.md` now shows `v0.1.15` in the main version marker,
- `README.md` repository snapshot now includes new U4-B1 docs/ADR/review/traceability artifacts and `verzola-proxy/src/tls/`,
- `README.md` quick-start targeted tests include `tls_negotiation_classification`,
- `README.md` roadmap/next-actions now point to `Unit U4 Bolt U4-B2`,
- `CHANGELOG.md` now includes a `v0.1.15` entry summarizing U4-B1 and manual cleanup candidates.

## 5) Manual Cleanup Visibility (Non-Destructive)

Updated:

- `CHANGELOG.md`

What was documented:

- currently present cargo build outputs and prior probe artifacts under `repo/` and `verzola-proxy/`,
- task-specific temporary directories created during test execution troubleshooting (`.tmp-tests/`, `codex-target-u4b1/`, `.codex-test-write/`),
- all listed under `### For Deletion` for manual cleanup only.

Important note:

- No files were deleted as part of this task.

## 6) Other Markdown Files

Not updated:

- `SECURITY.md`
- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`

Why:

- U4-B1 adds a local TLS classification utility and related documentation, but does not change contribution workflow, disclosure policy, or community conduct processes.

## Traceability Links

- Milestone source of truth:
  - `REQUIREMENTS.md`
- Classification implementation:
  - `verzola-proxy/src/tls/mod.rs`
  - `verzola-proxy/src/lib.rs`
- Classification tests:
  - `verzola-proxy/tests/tls_negotiation_classification.rs`
- Bolt artifacts:
  - `docs/tls-negotiation-classification-reference.md`
  - `docs/adr/0010-u4-b1-negotiated-group-classification.md`
  - `docs/reviews/u4-b1-crypto-classification-review.md`
  - `docs/bolts/u4-b1-traceability.md`

## Validation Notes

Validation command:

- `cargo test` (run in `verzola-proxy`)

Observed result:

- all suites passed, including `tls_negotiation_classification` (`3` tests, `0` failures in suite) and existing inbound/outbound regression suites.

Validation run date:

- `2026-02-23`
