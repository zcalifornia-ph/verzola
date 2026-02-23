# Policy Reporting CLI (`verzolactl report`)

This document defines the policy reporting workflow delivered in Unit U3 / Bolt U3-B3.

## Command Surface

Generate a posture report from a policy file:

```powershell
python -m verzola_control report <policy-file.yaml> --environment dev --format text --output -
```

Options:

- `--environment`: `dev`, `staging`, `prod`
- `--format`: `text`, `json`
- `--output`: destination file path, or `-` for stdout
- `--no-strict`: disable unknown-field rejection before report generation

## Validation Behavior

- `report` is validation-gated, the same as `validate` and `render`.
- Invalid files return exit code `1` and field-level diagnostics.
- Successful reports return exit code `0`.

## Report Sections

Text and JSON reports both include:

- policy file metadata (`policy_file`, `schema_version`, `environment`, `strict_mode`)
- posture summary:
  - inbound/outbound default mode and plaintext flags
  - domain override counts by mode
  - `require-pq` coverage count
  - DNS TXT hint status
- domain override details (deterministically sorted)
- detected gaps with severity and recommendations

## Severity Levels

- `critical`: immediate policy risk that can weaken transport guarantees.
- `warning`: policy posture issue that may violate expected hardening intent.
- `info`: advisory signal for operational tuning and policy precision.

## Gap Codes (Current)

- `U3R-001`: inbound plaintext allowed.
- `U3R-002`: outbound plaintext allowed.
- `U3R-003`: outbound default mode remains `opportunistic`.
- `U3R-004`: no domains currently use `require-pq`.
- `U3R-005`: domain override uses `opportunistic`.
- `U3R-006`: `require-pq` domain uses `defer` mismatch behavior.
- `U3R-007`: DNS TXT capability hints are disabled.

## Operator Workflow

1. Edit policy file.
2. Run `validate`.
3. Run `render` for deployment artifact.
4. Run `report` to inspect posture and detected gaps.
5. Tighten policy where critical/warning gaps are unacceptable.
6. Commit policy + generated artifacts + report evidence together for review.

