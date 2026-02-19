# U2-B2 Message Safety Regression Review (`250/4xx` Contract)

## Scope

Review artifact for `Unit U2 / Bolt U2-B2: Delivery Status Contract`.

## Checks Performed

- Status normalization behavior:
  - `RCPT` success maps to deterministic `250`,
  - final DATA success maps to deterministic `250`,
  - remote `4xx`/`5xx` delivery outcomes map to retry-safe `451`.
- Retry-safety contract:
  - temporary defer responses preserve Postfix queue/retry control.
- Data flow integrity:
  - `DATA` command still requires remote `3xx` before payload relay.
- Regression guard:
  - existing inbound + outbound orchestration tests remain green under normalized status mapping.

## Evidence

- Test command:
  - `cargo test`
- Relevant passing suites:
  - `outbound_orchestration` (2 passed)
  - `outbound_status_contract` (2 passed)
  - inbound suites (`inbound_forwarder`, `inbound_policy_telemetry`, `inbound_starttls`) remain passing

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-REL-02`:
  - transient and policy/defer-required outcomes return `4xx` to preserve queue semantics.
- Risk register (`Incorrect 250/4xx relay semantics`):
  - mitigated via explicit contract tests and deterministic mapping matrix.

## Residual Risks

- Permanent upstream failures are currently deferred (`451`) for safety, which may increase retry volume until richer policy/error taxonomy is introduced.
- Recipient-domain mixing remains intentionally constrained from U2-B1 scope.

## Sign-off

- Engineering regression review: complete for U2-B2 scope.
- Human message-safety sign-off: required before closing Unit U2.
