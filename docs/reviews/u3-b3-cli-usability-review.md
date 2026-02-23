# U3-B3 Usability Review (Policy Reports and CLI UX)

## Scope

Review artifact for `Unit U3 / Bolt U3-B3: Policy Reports and CLI UX`.

## Checks Performed

- Command-surface review:
  - `verzolactl validate` remains unchanged and compatible.
  - `verzolactl render` remains unchanged and compatible.
  - `verzolactl report` added with consistent option style (`--environment`, `--output`, `--no-strict`).
- Report readability review:
  - text format includes clear sections for defaults, domain posture, and detected gaps.
  - each gap includes severity, scope, and recommended operator action.
- Artifact workflow review:
  - JSON format supports CI ingestion and evidence capture.
  - file output workflow mirrors existing `render` UX.
- Determinism review:
  - domain override ordering and gap ordering are stable.

## Evidence

- Test command:
  - `python -B -m unittest discover -s tests -v` (run in `verzola-control`)
- Result:
  - passed (`20` tests, `0` failures).
- Report-specific coverage:
  - report JSON structure and severity assertions,
  - deterministic report equivalence across YAML/TOML,
  - CLI success/failure paths,
  - CLI output-file path using sample repo layout.

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-UX-01`:
  - report findings include scope and actionable recommendations.
- `NFR-CMP-01`:
  - report workflow and semantics are now versioned and documented.
- Risk register (`Policy misconfiguration by operators`):
  - policy posture and detected-gap report adds a second operator feedback layer after validation.

## Residual Risks

- Severity and gap rules are static and may need tuning as runtime policy features from later units land.
- Report signals are advisory and do not override runtime SMTP behavior.

## Sign-off

- CLI usability review: complete for U3-B3 scope.
- Human usability sign-off: captured as complete with Unit U3 closure.

