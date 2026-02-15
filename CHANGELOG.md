# Changelog

Status: pre-alpha (docs/spec complete, implementation in progress).

## v0.1.0

### Added or Changed
- Updated `README.md` version marker to `v0.1.0` and added explicit pre-alpha status messaging.
- Added root governance docs: `SECURITY.md`, `CONTRIBUTING.md`, and `CODE_OF_CONDUCT.md`.
- Updated `CHANGELOG.md` and `REQUIREMENTS.md` wording to align release expectations with the `0.x` pre-release cycle.

## v0.0.10

### Added or Changed
- Added learning module `learn/u1-b1-inbound-starttls-study-guide.md` to teach Unit U1 / Bolt U1 implementation details (mental model, code walkthrough, tests, drills, and debugging playbook).
- Updated `README.md` version marker from `v0.0.9` to `v0.0.10`.
- Updated `README.md` repository tree, quick-start flow, and progress notes to include the new learning asset under `learn/`.

## v0.0.9

### Added or Changed
- Completed Unit U1 / Bolt U1-B1 in `REQUIREMENTS.md` by checking all subtasks and marking the bolt as done with dated acceptance evidence.
- Installed and validated local Rust toolchain prerequisites for this workspace (`rustc 1.93.1`, `cargo 1.93.1`) and Windows MSVC Build Tools dependency for Rust linking.
- Executed acceptance run for `verzola-proxy` and confirmed inbound STARTTLS integration tests pass (`3 passed; 0 failed`).
- Updated `README.md` version marker from `v0.0.8` to `v0.0.9` and refreshed status/progress/next-actions text to reflect the completed U1-B1 milestone.

## v0.0.8

### Added or Changed
- Added initial `verzola-proxy` crate scaffold for Unit U1 / Bolt U1-B1, including `src/main.rs`, `src/lib.rs`, and inbound listener implementation under `src/inbound/mod.rs`.
- Added inbound STARTTLS integration tests in `verzola-proxy/tests/inbound_starttls.rs` covering success flow, temporary TLS failure mapping (`454`), and protocol-order enforcement.
- Added inbound implementation documentation and traceability artifacts:
  - `docs/inbound-listener.md`
  - `docs/adr/0001-u1-b1-listener-starttls-state-machine.md`
  - `docs/reviews/u1-b1-security-interoperability.md`
  - `docs/bolts/u1-b1-traceability.md`
- Updated `REQUIREMENTS.md` Unit U1 / Bolt U1-B1 section with a dated note describing current acceptance-run blocker in this environment (missing `cargo`/`rustc`).
- Updated `README.md` version marker from `v0.0.7` to `v0.0.8` and refreshed status/next-step guidance to reflect the new inbound implementation baseline.

## v0.0.7

### Added or Changed
- Rewrote `README.md` with a full VERZOLA project narrative and architecture blueprint.
- Added explicit scope sections for what VERZOLA is and is not, including deployment modes and policy model.
- Added detailed transport design sections covering inbound fronting, outbound relay semantics, and Postfix wiring.
- Added observability, security threat model, repository plan, demo plan, and phased delivery roadmap content.
- Added draft `verzolactl` policy YAML and draft Postfix `main.cf` / `master.cf` snippets for implementation guidance.
- Updated `repo/images/verzola-screen.png` screenshot asset.
- Removed obsolete `repo/images/logo.png` project image asset.
- Updated README version marker from `v0.0.6` to `v0.0.7`.

## v0.0.6

### Added or Changed
- Added root project documentation files: `README.md` and `CHANGELOG.md`
- Added GitHub issue templates under `.github/ISSUE_TEMPLATE/`
- Added project image assets under `repo/images/`
- Replaced root `LICENSE` with `LICENSE.txt` and preserved Apache 2.0 terms

## v0.0.5

### Added or Changed
- Moved repository images from `images/` to `repo/images/`
- Renamed screenshot asset to `repo/images/verzola-screen.png`
- Updated `README.md` image references to preserve rendering

## v0.0.4

### Added or Changed
- Added this changelog

### Removed
- forked README and replaced with BLANK_README
