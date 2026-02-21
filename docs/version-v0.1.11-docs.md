# Version v0.1.11 Documentation

## Title
Unit U3-B1 Control-Plane Validation Baseline

## Quick Diagnostic Read

This version delivers the first control-plane implementation slice (`Unit U3 / Bolt U3-B1`) and syncs root governance docs with that new scope.

Primary outcomes:

- `verzola-control` now exists with a policy schema model, parser support (`YAML`/`TOML`), strict validation engine, and CLI entrypoint (`verzolactl validate`).
- Edge-case and malformed-policy tests are now in place for the control-plane validation path.
- Root docs (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`) now reflect U3-B1 completion status and validation workflow.

## One-Sentence Objective

Establish a deterministic, test-covered policy validation baseline for `verzolactl` and make project-level documentation consistent with the new U3-B1 implementation reality.

## Scope of This Version

This version includes:

- new Python control-plane source code under `verzola-control/`,
- control-plane test coverage,
- Unit U3-B1 ADR/review/traceability and schema reference docs,
- root document synchronization for release/version and contributor/security guidance.

## Detailed Changes

## 1) Control-Plane Package Scaffold

Added:

- `verzola-control/pyproject.toml`
- `verzola-control/verzola_control/__init__.py`
- `verzola-control/verzola_control/__main__.py`
- `verzola-control/verzola_control/cli.py`

This establishes a first runnable control-plane package and command surface.

## 2) Policy Schema, Parsing, and Strict Validation

Added:

- `verzola-control/verzola_control/policy/model.py`
- `verzola-control/verzola_control/policy/parser.py`
- `verzola-control/verzola_control/validate/engine.py`

Key implementation outcomes:

- explicit schema model for listener/domain/capability policy fields,
- strict unknown-field rejection mode,
- actionable diagnostics with file path + field path + correction suggestion,
- domain normalization and duplicate-rule conflict checks.

## 3) Control-Plane Test Coverage

Added:

- `verzola-control/tests/test_validate_engine.py`

Coverage includes:

- valid YAML path,
- valid TOML path,
- malformed YAML parse failure,
- unknown field rejection,
- unknown policy mode handling,
- normalized-domain conflict detection,
- invalid `on_mismatch` usage handling,
- CLI success/failure exit behavior.

## 4) U3-B1 Documentation Artifacts

Added:

- `docs/policy-schema-reference.md`
- `docs/adr/0007-u3-b1-schema-validation-engine.md`
- `docs/reviews/u3-b1-maintainability-review.md`
- `docs/bolts/u3-b1-traceability.md`

These provide contract, rationale, maintainability checks, and acceptance traceability for U3-B1.

## 5) Root Documentation and Governance Sync

Updated:

- `README.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `REQUIREMENTS.md`

Notable sync points:

- `README.md` now shows `v0.1.11`,
- repository snapshot includes U3-B1 artifacts plus this file,
- quick start now includes control-plane tests and CLI validation example,
- `REQUIREMENTS.md` marks U3-B1 complete with dated evidence,
- contributor/security docs now reflect implemented control-plane validation scope.

## Traceability Links

- Milestone source of truth:
  - `REQUIREMENTS.md`
- Control-plane implementation:
  - `verzola-control/verzola_control/policy/model.py`
  - `verzola-control/verzola_control/policy/parser.py`
  - `verzola-control/verzola_control/validate/engine.py`
  - `verzola-control/verzola_control/cli.py`
- Validation tests:
  - `verzola-control/tests/test_validate_engine.py`
- Bolt artifacts:
  - `docs/adr/0007-u3-b1-schema-validation-engine.md`
  - `docs/reviews/u3-b1-maintainability-review.md`
  - `docs/bolts/u3-b1-traceability.md`

## Validation Notes

Validation command:

- `python -B -m unittest discover -s tests -v` (run in `verzola-control`)

Observed result:

- all suites passed (`9` tests, `0` failures).

Validation run date:

- `2026-02-21`
