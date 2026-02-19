a# Version v0.1.7 Documentation

## Title
Unit U2-B1 Outbound Orchestration Implementation and Documentation Sync

## Quick Diagnostic Read

This version delivers the first outbound data-plane implementation slice and aligns root documentation with that new reality.

Primary outcomes:

- outbound relay orchestration now exists in code with integration tests,
- U2-B1 is completed in `REQUIREMENTS.md`,
- root documentation and governance docs now reflect outbound scope entry.

## One-Sentence Objective

Ship Unit U2 Bolt U2-B1 as a traceable, test-covered outbound orchestration increment and keep repository documentation synchronized with implementation status.

## Scope of This Version

This version includes both code and documentation changes:

- outbound module and tests in `verzola-proxy`,
- requirements and README milestone/status updates,
- new ADR/review/traceability docs for U2-B1,
- changelog and security/contributor guidance updates.

## Detailed Changes

## 1) Outbound Session Orchestration (U2-B1)

Added:

- `verzola-proxy/src/outbound/mod.rs`
- `verzola-proxy/tests/outbound_orchestration.rs`

Implemented behavior:

- Postfix-facing outbound listener and SMTP sequencing,
- recipient-domain extraction from `RCPT TO`,
- MX candidate resolution via trait seam (`MxResolver`),
- deterministic MX ordering and candidate failover attempts,
- remote session bootstrap checks (`banner`, `EHLO`, staged `MAIL`),
- temporary failure mapping (`451 4.4.0`) for retry-safe outcomes.

## 2) Requirements and Traceability

Updated:

- `REQUIREMENTS.md`:
  - marked `Bolt U2-B1` and all subtasks complete,
  - added completion date and acceptance evidence.

Added:

- `docs/adr/0004-u2-b1-outbound-session-orchestration.md`
- `docs/reviews/u2-b1-protocol-behavior-review.md`
- `docs/bolts/u2-b1-traceability.md`
- `docs/outbound-relay-configuration.md`

## 3) Root Documentation and Governance Alignment

Updated:

- `README.md`
  - version marker to `v0.1.7`,
  - repository snapshot includes outbound module/tests and U2-B1 docs,
  - quick-start includes outbound integration suite,
  - progress/validation notes updated for U2-B1 completion.
- `CHANGELOG.md`
  - added `v0.1.7` release notes and manual cleanup targets.
- `SECURITY.md`
  - current implemented scope now includes U2-B1 outbound orchestration.
- `CONTRIBUTING.md`
  - optional local validation commands include `outbound_orchestration`.

## Traceability Links

- Requirements:
  - `REQUIREMENTS.md`
- Outbound implementation:
  - `verzola-proxy/src/outbound/mod.rs`
  - `verzola-proxy/tests/outbound_orchestration.rs`
- U2-B1 documentation:
  - `docs/outbound-relay-configuration.md`
  - `docs/adr/0004-u2-b1-outbound-session-orchestration.md`
  - `docs/reviews/u2-b1-protocol-behavior-review.md`
  - `docs/bolts/u2-b1-traceability.md`

## Validation Notes

Acceptance validation command:

- `cargo test` (run in `verzola-proxy`)

Observed results for implemented scope:

- inbound suites: `2 + 4 + 3` tests passed,
- outbound orchestration suite: `2` tests passed.

Validation run date:

- `2026-02-18`
