# Bolt U3-B1 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: schema and validation engine for policy-as-code control plane.
- Subtasks:
  - Design: schema model and error taxonomy.
  - Implement: parser + validator with strict mode.
  - Test: malformed/edge-case fixture suite.
  - Docs: policy schema reference.
  - Review: maintainability review for future policy expansion.

## Context Summary

- The repository had no `verzola-control` implementation before this bolt.
- Unit U3 acceptance requires actionable validation failures and deterministic schema behavior.
- Policy modes already established in project docs needed executable schema enforcement.
- Domain-level overrides required normalized matching to prevent duplicate/conflicting rules.
- Strict mode was required to catch unknown fields early and avoid config drift.
- Existing project direction expects `verzolactl validate` as a baseline Phase 0 capability.
- Diagnostics needed to be operator-friendly (`file path + field path + correction`) per NFR UX expectations.
- U3-B1 had to stay scoped: validation first, while renderer/report workflows are deferred to U3-B2/U3-B3.

## File-Level Plan and Outputs

- Added:
  - `verzola-control/pyproject.toml`
  - `verzola-control/verzola_control/__init__.py`
  - `verzola-control/verzola_control/__main__.py`
  - `verzola-control/verzola_control/cli.py`
  - `verzola-control/verzola_control/policy/__init__.py`
  - `verzola-control/verzola_control/policy/model.py`
  - `verzola-control/verzola_control/policy/parser.py`
  - `verzola-control/verzola_control/validate/__init__.py`
  - `verzola-control/verzola_control/validate/engine.py`
  - `verzola-control/tests/test_validate_engine.py`
  - `docs/adr/0007-u3-b1-schema-validation-engine.md`
  - `docs/policy-schema-reference.md`
  - `docs/reviews/u3-b1-maintainability-review.md`
  - `docs/bolts/u3-b1-traceability.md`

## Acceptance Run

- Command:
  - `python -B -m unittest discover -s tests -v`
- Run location:
  - `verzola-control`
- Result:
  - passed (`9` tests, `0` failures).
- Completed:
  - `2026-02-21`

## NFR/Risk Notes

- UX:
  - validation diagnostics now include file path, field path, and suggestions.
- Security/reliability guardrail:
  - strict schema mode rejects unknown fields and invalid policy values before deployment.
- Risk mitigation:
  - policy-misconfiguration risk is reduced through deterministic validation and duplicate-domain detection.
