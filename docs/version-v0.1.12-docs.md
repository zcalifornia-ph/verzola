# Version v0.1.12 Documentation

## Title
Unit U3-B2 Deterministic Config Renderer

## Quick Diagnostic Read

This version completes `Unit U3 / Bolt U3-B2` by adding deterministic, environment-aware policy rendering to the control-plane package.

Primary outcomes:

- `verzolactl render` now exists and is validation-gated.
- Rendered effective config artifacts are deterministic and snapshot-testable.
- U3-B2 ADR/review/traceability and renderer semantics docs are now versioned.

## One-Sentence Objective

Turn validated policy files into reproducible, environment-aware effective config artifacts for proxy ingestion workflows.

## Scope of This Version

This version includes:

- a new renderer package in `verzola-control`,
- CLI render command support,
- render-focused test coverage,
- Unit U3-B2 documentation artifacts,
- root-document synchronization for milestone/validation status.

## Detailed Changes

## 1) Renderer Implementation

Added:

- `verzola-control/verzola_control/render/engine.py`
- `verzola-control/verzola_control/render/__init__.py`

Key implementation outcomes:

- intermediate representation dataclasses for rendered artifact structure,
- built-in environment profiles (`dev`, `staging`, `prod`),
- deterministic JSON rendering with canonical ordering.

## 2) CLI Extension (`verzolactl render`)

Updated:

- `verzola-control/verzola_control/cli.py`

New behavior:

- `render` subcommand with `--environment` and `--output`,
- strict validation gate before artifact generation,
- stdout or file output workflows.

## 3) Render Validation Coverage

Added:

- `verzola-control/tests/test_render_engine.py`

Coverage includes:

- snapshot assertion for deterministic renderer output,
- YAML/TOML equivalence determinism checks,
- environment-profile variance checks,
- CLI render success/failure behavior.

## 4) U3-B2 Documentation Artifacts

Added:

- `docs/policy-renderer-artifact-semantics.md`
- `docs/adr/0008-u3-b2-config-renderer.md`
- `docs/reviews/u3-b2-proxy-ingestion-compatibility-review.md`
- `docs/bolts/u3-b2-traceability.md`

## 5) Root Documentation Sync

Updated:

- `README.md`
- `CHANGELOG.md`
- `REQUIREMENTS.md`
- `CONTRIBUTING.md`
- `SECURITY.md`

Notable sync points:

- `README.md` now shows `v0.1.12`,
- `REQUIREMENTS.md` marks U3-B2 complete with dated evidence,
- contributor/security docs now include render workflow and updated scope.

## Traceability Links

- Milestone source of truth:
  - `REQUIREMENTS.md`
- Renderer implementation:
  - `verzola-control/verzola_control/render/engine.py`
  - `verzola-control/verzola_control/cli.py`
- Render tests:
  - `verzola-control/tests/test_render_engine.py`
- Bolt artifacts:
  - `docs/adr/0008-u3-b2-config-renderer.md`
  - `docs/reviews/u3-b2-proxy-ingestion-compatibility-review.md`
  - `docs/bolts/u3-b2-traceability.md`

## Validation Notes

Validation command:

- `python -B -m unittest discover -s tests -v` (run in `verzola-control`)

Observed result:

- all suites passed (`15` tests, `0` failures).

Validation run date:

- `2026-02-21`
