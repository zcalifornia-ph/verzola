# Bolt U3-B3 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: policy reports and CLI UX completion for Unit U3 control-plane workflows.
- Subtasks:
  - Design: report sections and severity levels.
  - Implement: CLI commands for validate/render/report.
  - Test: CLI integration tests with sample repos.
  - Docs: operator workflow from policy edit to deploy.
  - Review: usability review for actionable errors.

## Context Summary

- U3-B1 and U3-B2 delivered policy validation and deterministic rendering, but not operator-facing posture reporting.
- Unit U3 acceptance still required reports that summarize domain policy posture and detected gaps.
- Existing CLI ergonomics (`validate` and `render`) already established option patterns and error rendering.
- Documentation and changelog conventions required synchronized root-doc updates at bolt close.
- Report output needed both human-readable and machine-readable forms for operations and CI usage.
- Deterministic ordering remained necessary to keep report artifacts diff-friendly.

## File-Level Plan and Outputs

- Added:
  - `verzola-control/verzola_control/report/__init__.py`
  - `verzola-control/verzola_control/report/engine.py`
  - `verzola-control/tests/test_report_engine.py`
  - `docs/policy-reporting-cli.md`
  - `docs/adr/0009-u3-b3-policy-reporting-cli-ux.md`
  - `docs/reviews/u3-b3-cli-usability-review.md`
  - `docs/bolts/u3-b3-traceability.md`
  - `docs/version-v0.1.13-docs.md`
- Updated:
  - `verzola-control/verzola_control/cli.py`
  - `verzola-control/pyproject.toml`
  - `REQUIREMENTS.md`
  - `README.md`
  - `CHANGELOG.md`
  - `CONTRIBUTING.md`
  - `SECURITY.md`

## Acceptance Run

- Command:
  - `python -B -m unittest discover -s tests -v`
- Run location:
  - `verzola-control`
- Result:
  - passed (`20` tests, `0` failures).
- Completed:
  - `2026-02-21`

## NFR/Risk Notes

- UX:
  - report output now gives severity-tagged, scoped recommendations for policy hardening decisions.
- Compliance:
  - report workflow and documentation artifacts are now version-controlled and traceable.
- Risk mitigation:
  - misconfiguration risk is reduced by adding posture-level gap reporting on top of schema validation.

