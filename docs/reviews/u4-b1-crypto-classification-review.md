# U4-B1 Crypto Classification Review (Negotiation Result Classification)

## Scope

Review artifact for `Unit U4 / Bolt U4-B1: Negotiation Result Classification`.

## Checks Performed

- Classification safety:
  - hybrid groups (for example `X25519MLKEM768`) resolve to `pq`.
  - unknown/unmapped groups resolve to `none` (no optimistic inference).
- Parser robustness:
  - supports common metadata formats (`=`, `:`, spaced separators, key + next-token).
  - ignores non-group metadata (for example cipher-only lines).
- Determinism:
  - normalization removes case and punctuation variation before mapping.
- Future-policy safety:
  - conservative `none` default avoids silently weakening later `require-pq` decisions.

## Evidence

- Test command:
  - `cargo test`
- Run location:
  - `verzola-proxy`
- Relevant passing suites:
  - `tls_negotiation_classification` (3 passed)
  - `outbound_tls_policy` (6 passed)
  - `outbound_orchestration` (2 passed)
  - `outbound_status_contract` (2 passed)
  - inbound suites remain passing (regression guard)

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-SEC-02`:
  - negotiated-group classification now has deterministic outcomes for `pq/classical/none`.
- `NFR-CMP-01`:
  - implementation, ADR, review, and bolt traceability artifacts are version-controlled for auditability.
- Risk register (`Hybrid/PQ interop instability across MTAs/TLS stacks`):
  - ambiguous or unknown metadata is handled conservatively (`none`) while explicit PQ/hybrid markers are recognized.

## Residual Risks

- Numeric/draft group IDs without explicit name mapping remain `none` until the mapping table is expanded.
- Real TLS adapter wiring and per-session metadata capture are still broader Unit U4 work and continue in later bolts.

## Sign-off

- Engineering crypto-classification review: complete for U4-B1 scope.
- Human cryptography/security sign-off: required before closing Unit U4.
