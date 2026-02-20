# U2-B3 Downgrade Resistance Review (Outbound TLS Policy Application)

## Scope

Review artifact for `Unit U2 / Bolt U2-B3: Outbound TLS Policy Application`.

## Checks Performed

- Policy resolution order:
  - per-domain override precedence over global policy.
- Opportunistic behavior:
  - plaintext continues when STARTTLS is unavailable.
  - fallback path activates when STARTTLS negotiation fails.
- Strict policy behavior:
  - `require-tls` returns deterministic defer (`451 4.7.5`) on STARTTLS absence/failure.
- Retry-safety:
  - strict-policy failures are mapped to temporary defer so Postfix retains retry ownership.
- Config hygiene:
  - duplicate recipient-domain rules are rejected during config validation.

## Evidence

- Test command:
  - `cargo test`
- Relevant passing suites:
  - `outbound_orchestration` (2 passed)
  - `outbound_status_contract` (2 passed)
  - `outbound_tls_policy` (6 passed)
  - inbound suites remain passing (regression guard)

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-SEC-02`:
  - policy violations produce deterministic defer behavior for strict mode.
- `NFR-REL-02`:
  - policy failure paths return `4xx` and preserve Postfix queue/retry semantics.
- Risk register (`Hybrid/PQ interop instability`):
  - strict-vs-opportunistic handling is now explicit and testable while PQ mode remains feature-gated in later units.

## Residual Risks

- STARTTLS decision boundary is implemented, but full TLS adapter hardening remains a later milestone.
- `require-pq` handling and negotiated-group telemetry are out of scope for U2-B3 and tracked in Unit U4.

## Sign-off

- Engineering downgrade-resistance review: complete for U2-B3 scope.
- Human security sign-off: required before closing Unit U2.
