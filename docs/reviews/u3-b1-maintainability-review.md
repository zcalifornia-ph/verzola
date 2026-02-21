# U3-B1 Maintainability Review (Schema and Validation Engine)

## Scope

Review artifact for `Unit U3 / Bolt U3-B1: Schema and Validation Engine`.

## Checks Performed

- Package structure review:
  - clear separation between schema model (`policy/model.py`), parsing (`policy/parser.py`), validation (`validate/engine.py`), and CLI entrypoint (`cli.py`).
- Error taxonomy review:
  - parser errors converted into structured diagnostics,
  - schema errors reported with field-level paths and suggestions.
- Strictness review:
  - unknown fields are rejected in strict mode at all defined schema levels.
- Policy semantics review:
  - supported policy modes validated against README contract,
  - `on_mismatch` constrained to `require-pq` domain rules,
  - normalized-domain duplicate detection.
- Test coverage review:
  - valid YAML and TOML paths,
  - malformed YAML,
  - unknown field handling,
  - unknown policy value handling,
  - domain normalization conflict handling,
  - CLI success and failure exit paths.

## Evidence

- Test command:
  - `python -B -m unittest discover -s tests -v` (run in `verzola-control`)
- Result:
  - passed (`9` tests, `0` failures).

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-UX-01`:
  - diagnostics include file path + field path + suggested correction.
- `NFR-CMP-01`:
  - schema, validation, and review artifacts are versioned in repository docs/code.
- Risk register (`Policy misconfiguration by operators`):
  - strict schema checks and deterministic diagnostics reduce accidental unsafe policy drift.

## Residual Risks

- YAML parser intentionally supports project policy shape only, not full YAML feature set.
- CLI currently implements `validate` only; `render/report` remain for later bolts.

## Sign-off

- Engineering maintainability review: complete for U3-B1 scope.
- Human maintainability sign-off: required before closing Unit U3.
