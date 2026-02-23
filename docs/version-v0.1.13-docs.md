# Version v0.1.13 Documentation

## Title
Unit U3-B3 Policy Reports and CLI UX

## Quick Diagnostic Read

This version completes `Unit U3 / Bolt U3-B3` by adding policy posture reporting to `verzolactl`.

Primary outcomes:

- `verzolactl report` now exists with `text` and `json` outputs.
- Reports summarize policy posture and detected gaps with severity levels.
- Unit U3 is now complete in `REQUIREMENTS.md`.

## One-Sentence Objective

Give operators a deterministic, actionable CLI report that turns validated policy files into posture and gap evidence for deployment decisions.

## Scope of This Version

This version includes:

- a new report package in `verzola-control`,
- CLI report command support,
- report-focused test coverage,
- Unit U3-B3 documentation artifacts,
- root-document synchronization for Unit U3 closure.

## Detailed Changes

## 1) Report Implementation

Added:

- `verzola-control/verzola_control/report/engine.py`
- `verzola-control/verzola_control/report/__init__.py`

Key implementation outcomes:

- posture summary model for defaults/domains/capability hints,
- deterministic gap detection with severity (`critical`, `warning`, `info`),
- text and JSON report rendering.

## 2) CLI Extension (`verzolactl report`)

Updated:

- `verzola-control/verzola_control/cli.py`

New behavior:

- `report` subcommand with `--environment`, `--format`, and `--output`,
- strict/non-strict schema parity with existing commands,
- stdout or file output workflows for report artifacts.

## 3) Report Validation Coverage

Added:

- `verzola-control/tests/test_report_engine.py`

Coverage includes:

- report JSON structure and severity assertions,
- deterministic report equivalence across YAML/TOML policy inputs,
- CLI report success/failure behavior,
- CLI output-file flow against sample repo layout.

## 4) U3-B3 Documentation Artifacts

Added:

- `docs/policy-reporting-cli.md`
- `docs/adr/0009-u3-b3-policy-reporting-cli-ux.md`
- `docs/reviews/u3-b3-cli-usability-review.md`
- `docs/bolts/u3-b3-traceability.md`

## 5) Root Documentation Sync

Updated:

- `README.md`
- `CHANGELOG.md`
- `REQUIREMENTS.md`
- `CONTRIBUTING.md`
- `SECURITY.md`

Notable sync points:

- `README.md` now shows `v0.1.13`,
- `REQUIREMENTS.md` marks U3-B3 and Unit U3 complete with dated evidence,
- contributor/security docs now include report workflow and updated Unit U4-U6 focus.

## Traceability Links

- Milestone source of truth:
  - `REQUIREMENTS.md`
- Report implementation:
  - `verzola-control/verzola_control/report/engine.py`
  - `verzola-control/verzola_control/cli.py`
- Report tests:
  - `verzola-control/tests/test_report_engine.py`
- Bolt artifacts:
  - `docs/adr/0009-u3-b3-policy-reporting-cli-ux.md`
  - `docs/reviews/u3-b3-cli-usability-review.md`
  - `docs/bolts/u3-b3-traceability.md`

## Validation Notes

Validation command:

- `python -B -m unittest discover -s tests -v` (run in `verzola-control`)

Observed result:

- all suites passed (`20` tests, `0` failures).

Validation run date:

- `2026-02-21`

