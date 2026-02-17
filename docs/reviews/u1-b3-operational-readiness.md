# U1-B3 Operational Readiness Review (Inbound Policy and Telemetry)

## Scope

Review artifact for `Unit U1 / Bolt U1-B3: Inbound Policy Enforcement and Telemetry`.

## Checks Performed

- Policy enforcement:
  - `opportunistic` mode allows plaintext envelope flow after `EHLO`.
  - `require-tls` mode blocks plaintext `MAIL/RCPT/DATA` with deterministic `530 5.7.0`.
- Configuration safety:
  - invalid `require-tls` + `advertise_starttls=false` configuration fails listener startup.
- Telemetry schema:
  - STARTTLS attempts/failures and policy rejection counters are captured per session summary.
- Regression safety:
  - existing STARTTLS and loopback relay integration suites remain passing.

## Evidence

- Test command:
  - `cargo test`
- Relevant passing tests:
  - `opportunistic_policy_allows_plaintext_mail_flow`
  - `require_tls_policy_rejects_plaintext_until_starttls`
  - `require_tls_listener_config_requires_starttls_advertisement`
  - `telemetry_tracks_tls_failures_and_policy_rejections`
  - Existing U1-B1 and U1-B2 integration suites also passed in the same run.

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-SEC-02`:
  - deterministic policy violation outcomes implemented for inbound `require-tls`.
- `NFR-REL-02`:
  - temporary TLS/relay failures keep temporary-response semantics (`454`/`451`) to preserve sender retry behavior.
- `US-01` policy and observability criteria:
  - policy matrix behavior and telemetry schema assertions are test-covered.

## Residual Risks

- Session telemetry is currently in-memory/session-scoped; aggregate export paths are still pending Unit U5.
- `require-pq` inbound policy remains out of scope for this bolt and is tracked in Unit U4.

## Sign-off

- Engineering pre-review: complete for U1-B3 scope.
- Human SRE sign-off: required before closing Unit U1 as fully operational.
