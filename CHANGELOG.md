# Changelog

Status: pre-alpha (docs/spec complete, implementation in progress).

## v0.1.11

### Added or Changed
- Completed Unit U3 / Bolt U3-B1 in `REQUIREMENTS.md` with checked subtasks, completion date, and acceptance evidence.
- Added control-plane package scaffold and CLI entrypoint under `verzola-control/`:
  - `verzola-control/pyproject.toml`
  - `verzola-control/verzola_control/__main__.py`
  - `verzola-control/verzola_control/cli.py`
- Added policy schema/domain model and parser modules:
  - `verzola-control/verzola_control/policy/model.py`
  - `verzola-control/verzola_control/policy/parser.py`
- Added strict validation engine with actionable diagnostics:
  - `verzola-control/verzola_control/validate/engine.py`
- Added malformed/edge-case + CLI validation tests:
  - `verzola-control/tests/test_validate_engine.py`
- Added Unit U3-B1 documentation and traceability artifacts:
  - `docs/policy-schema-reference.md`
  - `docs/adr/0007-u3-b1-schema-validation-engine.md`
  - `docs/reviews/u3-b1-maintainability-review.md`
  - `docs/bolts/u3-b1-traceability.md`
- Updated `README.md` version marker from `v0.1.10` to `v0.1.11` and synchronized quick start, repository snapshot, progress notes, and validation notes for U3-B1.
- Updated `CONTRIBUTING.md` local validation workflow to include control-plane `unittest` coverage and `verzolactl validate` examples.
- Updated `SECURITY.md` scope/limitations to include implemented Unit U3-B1 policy validation surface while keeping remaining U3-U6 scope explicit.
- Revalidated control-plane scope on `2026-02-21` with `python -B -m unittest discover -s tests -v` in `verzola-control` (`9` passed; `0` failed).
- Added detailed version documentation at `docs/version-v0.1.11-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `verzola-control/.tmp-tests/`
  - `verzola-proxy/target/`
  - `verzola-proxy/target_ci_test/`
  - `verzola-proxy/target_ci1a5BY6/`
  - `verzola-proxy/target_u1_b3/`
  - `verzola-proxy/target_u1_b38vm3eI/`
  - `verzola-proxy/target_u1_b3bfvozlA/`
  - `verzola-proxy/target_u2_b1/`
  - `verzola-proxy/target_u2_b1wSSldO/`
  - `verzola-proxy/target_u2_b3_rebuild/`
  - `verzola-proxy/target_u2_b3_rebuildrMqWaL/`
  - `verzola-proxy/target-u2b2SXi4dh/`
  - `verzola-proxy/targetnOSdwx/`
  - `verzola-proxy/tmp-check/`
  - `verzola-proxy/temp_test_dir/`
  - `verzola-proxy/rmetaB2f4iR/`
  - `verzola-proxy/rmetancZsdL/`
  - `verzola-proxy/rustc_probe.rs`
  - `verzola-proxy/rustc_probe.rustc_probe.c67070f154ac956c-cgu.0.rcgu.o`
  - `verzola-proxy/rename_probe.tmp`
  - `repo/target_ci/`
  - `repo/target_ci_u2_b3/`
  - `repo/target_ci_u2_b3RfMHAM/`

## v0.1.10

### Added or Changed
- Updated `README.md` version marker from `v0.1.9` to `v0.1.10` and aligned roadmap checkboxes with current milestone reality (`Phase 1` and `Phase 2` complete).
- Updated `README.md` immediate next actions to call out pending production TLS adapter wiring for both inbound and outbound paths.
- Updated `CONTRIBUTING.md` validation guidance to include outbound targeted suites for `outbound_status_contract` and `outbound_tls_policy`.
- Updated `SECURITY.md` scope/limitations to reflect completed Unit U2 (`U2-B1`, `U2-B2`, `U2-B3`) and to map pending security surface to Units U3-U6.
- Revalidated implemented proxy scope on `2026-02-20` with `cargo test` in `verzola-proxy` (`2 + 4 + 3 + 2 + 2 + 6` integration tests passed; `0` failed).
- Added detailed version documentation at `docs/version-v0.1.10-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `verzola-proxy/target/`
  - `verzola-proxy/target_ci_test/`
  - `verzola-proxy/target_ci1a5BY6/`
  - `verzola-proxy/target_u1_b3/`
  - `verzola-proxy/target_u1_b38vm3eI/`
  - `verzola-proxy/target_u1_b3bfvozlA/`
  - `verzola-proxy/target_u2_b1/`
  - `verzola-proxy/target_u2_b1wSSldO/`
  - `verzola-proxy/target_u2_b3_rebuild/`
  - `verzola-proxy/target_u2_b3_rebuildrMqWaL/`
  - `verzola-proxy/target-u2b2SXi4dh/`
  - `verzola-proxy/targetnOSdwx/`
  - `verzola-proxy/tmp-check/`
  - `verzola-proxy/temp_test_dir/`
  - `verzola-proxy/rmetaB2f4iR/`
  - `verzola-proxy/rmetancZsdL/`
  - `verzola-proxy/rustc_probe.rs`
  - `verzola-proxy/rustc_probe.rustc_probe.c67070f154ac956c-cgu.0.rcgu.o`
  - `verzola-proxy/rename_probe.tmp`
  - `repo/target_ci/`
  - `repo/target_ci_u2_b3/`
  - `repo/target_ci_u2_b3RfMHAM/`

## v0.1.9

### Added or Changed
- Completed Unit U2 / Bolt U2-B3 in `REQUIREMENTS.md` with checked subtasks, completion date, and acceptance evidence.
- Marked Unit U2 acceptance criteria and deliverables complete in `REQUIREMENTS.md` after U2-B3 validation.
- Finalized outbound TLS policy application in `verzola-proxy/src/outbound/mod.rs`:
  - global/per-domain policy model (`opportunistic`, `require-tls`),
  - STARTTLS capability detection and opportunistic fallback path,
  - strict policy defer mapping (`451 4.7.5`) for downgrade resistance,
  - policy/fallback summary fields for outbound session telemetry.
- Fixed outbound test suite compilation for new config fields in:
  - `verzola-proxy/tests/outbound_orchestration.rs`
  - `verzola-proxy/tests/outbound_status_contract.rs`
- Added outbound TLS policy integration coverage in `verzola-proxy/tests/outbound_tls_policy.rs` (6 tests covering mode behavior, fallback, strict defers, per-domain overrides, and duplicate-rule validation).
- Updated outbound operator docs in `docs/outbound-relay-configuration.md` with policy evaluation order, downgrade/defer matrix, and validation commands.
- Added U2-B3 traceability artifacts:
  - `docs/adr/0006-u2-b3-outbound-tls-policy-application.md`
  - `docs/reviews/u2-b3-downgrade-resistance-review.md`
  - `docs/bolts/u2-b3-traceability.md`
- Updated `README.md` to `v0.1.9` and synchronized repository snapshot/progress/next-actions for completed Unit U2.
- Added detailed version documentation at `docs/version-v0.1.9-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `verzola-proxy/target/`
  - `repo/target_ci_u2_b3/`
  - `repo/target_ci_u2_b3RfMHAM/`
  - `verzola-proxy/target_u2_b3_rebuild/`
  - `verzola-proxy/target_u2_b3_rebuildrMqWaL/`
  - `verzola-proxy/.tmpArurTo.temp-archive/`
  - `verzola-proxy/rmetaB2f4iR/`
  - `verzola-proxy/rmetancZsdL/`
  - `verzola-proxy/rustc_probe.rs`
  - `verzola-proxy/rustc_probe.rustc_probe.c67070f154ac956c-cgu.0.rcgu.o`
  - `verzola-proxy/rename_probe.tmp`

## v0.1.8

### Added or Changed
- Completed Unit U2 / Bolt U2-B2 in `REQUIREMENTS.md` with checked subtasks, completion date, and acceptance evidence.
- Added deterministic outbound delivery status mapping in `verzola-proxy/src/outbound/mod.rs` so Postfix-facing outcomes normalize to `250` on confirmed acceptance and retry-safe `451` on defer paths.
- Updated outbound integration expectations in `verzola-proxy/tests/outbound_orchestration.rs` to match normalized status contract behavior.
- Added outbound status-contract integration coverage in `verzola-proxy/tests/outbound_status_contract.rs` for transient and permanent upstream outcome classification.
- Updated operator-facing outbound contract documentation in `docs/outbound-relay-configuration.md`, including mapping matrix and troubleshooting guidance.
- Added U2-B2 traceability artifacts:
  - `docs/adr/0005-u2-b2-delivery-status-contract.md`
  - `docs/reviews/u2-b2-message-safety-regression-review.md`
  - `docs/bolts/u2-b2-traceability.md`
- Updated `README.md` to `v0.1.8` and synced repository snapshot to include the new version documentation file.
- Added detailed version documentation at `docs/version-v0.1.8-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `verzola-proxy/target/`
  - `verzola-proxy/target_ci_test/`
  - `verzola-proxy/target_ci1a5BY6/`
  - `verzola-proxy/target_u1_b3/`
  - `verzola-proxy/target_u1_b38vm3eI/`
  - `verzola-proxy/target_u1_b3bfvozlA/`
  - `verzola-proxy/target_u2_b1/`
  - `verzola-proxy/target_u2_b1wSSldO/`
  - `verzola-proxy/target-u2b2SXi4dh/`
  - `verzola-proxy/targetnOSdwx/`
  - `verzola-proxy/tmp-check/`

## v0.1.7

### Added or Changed
- Completed Unit U2 / Bolt U2-B1 in `REQUIREMENTS.md` with checked subtasks, completion date, and acceptance evidence.
- Added outbound session orchestration implementation in `verzola-proxy/src/outbound/mod.rs` and exported it through `verzola-proxy/src/lib.rs`.
- Added outbound orchestration integration coverage in `verzola-proxy/tests/outbound_orchestration.rs` (MX failover success path + temporary-failure path).
- Added U2-B1 documentation artifacts:
  - `docs/outbound-relay-configuration.md`
  - `docs/adr/0004-u2-b1-outbound-session-orchestration.md`
  - `docs/reviews/u2-b1-protocol-behavior-review.md`
  - `docs/bolts/u2-b1-traceability.md`
- Updated `README.md` to `v0.1.7`, synced repository snapshot/quick-start/progress notes for U2-B1, and advanced next actions to U2-B2.
- Updated `SECURITY.md` and `CONTRIBUTING.md` so current scope and local validation commands include the new outbound orchestration slice.
- Added detailed version documentation at `docs/version-v0.1.7-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `repo/target_ci/`
  - `verzola-proxy/target/`
  - `verzola-proxy/target_ci_test/`
  - `verzola-proxy/target_ci1a5BY6/`
  - `verzola-proxy/target_u1_b3/`
  - `verzola-proxy/target_u1_b38vm3eI/`
  - `verzola-proxy/target_u1_b3bfvozlA/`
  - `verzola-proxy/target_u2_b1/`
  - `verzola-proxy/target_u2_b1wSSldO/`
  - `verzola-proxy/targetnOSdwx/`
  - `verzola-proxy/temp_test_dir/`

## v0.1.6

### Added or Changed
- Updated `README.md` version marker from `v0.1.5` to `v0.1.6`.
- Replaced the outdated `README.md` repository tree with a current repository snapshot and an explicit planned-expansion note aligned to `REQUIREMENTS.md` Units U2-U6.
- Updated `README.md` quick-start and roadmap sections to include explicit current inbound test coverage (`inbound_starttls`, `inbound_forwarder`, `inbound_policy_telemetry`) plus a dated validation note.
- Updated `CONTRIBUTING.md` with documentation-sync expectations (`REQUIREMENTS.md`, `README.md`, `CHANGELOG.md`) and explicit inbound test command guidance.
- Updated `SECURITY.md` to document current pre-alpha security scope/limitations, including pending production TLS adapter work and planned U2-U6 security surface.
- Added detailed version documentation at `docs/version-v0.1.6-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `repo/target_ci/`
  - `verzola-proxy/target/`
  - `verzola-proxy/target_ci_test/`
  - `verzola-proxy/target_ci1a5BY6/`
  - `verzola-proxy/target_u1_b3/`
  - `verzola-proxy/target_u1_b38vm3eI/`
  - `verzola-proxy/target_u1_b3bfvozlA/`
  - `verzola-proxy/targetnOSdwx/`
  - `verzola-proxy/temp_test_dir/`

## v0.1.5

### Added or Changed
- Added learning module `learn/u1-b3-inbound-policy-telemetry-study-guide.md` with a practical walkthrough of Unit U1 / Bolt U1-B3 (policy model, telemetry schema, deterministic SMTP mappings, tests, and drills).
- Updated `README.md` version marker from `v0.1.4` to `v0.1.5`.
- Updated `README.md` learning references in the repository tree, quick-start section, and roadmap learning note to include the new U1-B3 guide.
- Added detailed version documentation at `docs/version-v0.1.5-docs.md`.

### For Deletion
- Build/test artifacts currently present in workspace (left intentionally for manual cleanup):
  - `verzola-proxy/target/debug/**`
  - `repo/target_ci/debug/**`
  - `verzola-proxy/target_u1_b3*/`
  - `verzola-proxy/temp_test_dir/`

## v0.1.4

### Added or Changed
- Completed Unit U1 / Bolt U1-B3 in `REQUIREMENTS.md` with policy/telemetry subtasks checked, dated completion, and acceptance evidence.
- Added inbound policy enforcement + telemetry schema in `verzola-proxy/src/inbound/mod.rs`:
  - explicit `InboundTlsPolicy` (`opportunistic` / `require-tls`),
  - deterministic `530 5.7.0` mapping for plaintext commands under `require-tls`,
  - session telemetry counters for STARTTLS attempts/failures and policy/relay outcomes.
- Added integration coverage in `verzola-proxy/tests/inbound_policy_telemetry.rs` for policy matrix and telemetry assertions.
- Updated inbound docs/review/traceability artifacts:
  - `docs/inbound-policy-telemetry.md`
  - `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`
  - `docs/reviews/u1-b3-operational-readiness.md`
  - `docs/bolts/u1-b3-traceability.md`
- Updated `README.md` version marker from `v0.1.3` to `v0.1.4`, inbound references, progress note, and immediate next actions.

### For Deletion
- Build/test artifacts generated during acceptance run (left intentionally for manual cleanup):
  - `verzola-proxy/target/debug/**`
  - `repo/target_ci/debug/**`
  - `verzola-proxy/target_u1_b3*/`

## v0.1.3

### Added or Changed
- Updated `README.md` version marker from `v0.1.2` to `v0.1.3`.
- Expanded `README.md` quick-start prerequisites to include `cargo --version` verification before running tests.
- Added Windows remediation steps in `README.md` for `'cargo' is not recognized`, including temporary session PATH fix and persistent user PATH update guidance.

### For Deletion
- None from this task context (documentation-only changes; no new build artifacts generated by this update).

## v0.1.2

### Added or Changed
- Added learning module `learn/u1-b2-streaming-forwarder-study-guide.md` with a practical walkthrough of Unit U1 / Bolt U1-B2 (streaming relay architecture, tests, debugging, and drills).
- Updated `README.md` version marker from `v0.1.1` to `v0.1.2`.
- Updated `README.md` learning references in the repository tree, quick-start section, and roadmap learning note to include the new U1-B2 guide.

### For Deletion
- None from this task context (no new build artifacts were generated).

## v0.1.1

### Added or Changed
- Completed Unit U1 / Bolt U1-B2 in `REQUIREMENTS.md` with streaming command/DATA relay to loopback Postfix and acceptance evidence.
- Extended inbound proxy relay support in `verzola-proxy/src/inbound/mod.rs`, including bounded DATA streaming and temporary failure mapping for relay outages.
- Added integration coverage for large-message relay and concurrent-session relay in `verzola-proxy/tests/inbound_forwarder.rs`.
- Added U1-B2 documentation artifacts:
  - `docs/inbound-postfix-integration.md`
  - `docs/adr/0002-u1-b2-streaming-forwarder.md`
  - `docs/bolts/u1-b2-traceability.md`
  - `docs/reviews/u1-b2-performance-review.md`
- Updated `README.md` quick-start/progress/next-action text to reflect U1-B2 completion and the next U1-B3 milestone.

### For Deletion
- `repo/target_ci/` (temporary cargo target directory created during test execution troubleshooting).
- `verzola-proxy/target_ci1a5BY6/` (temporary cargo target directory artifact).
- `verzola-proxy/target_ci_test/` (temporary cargo target directory artifact).
- `verzola-proxy/target/debug/**` build outputs modified/generated by `cargo test` runs (fingerprints, deps, incremental objects, binaries, and pdb files).

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
