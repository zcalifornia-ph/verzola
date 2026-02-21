# U3-B2 Compatibility Review (Config Renderer -> Proxy Ingestion)

## Scope

Review artifact for `Unit U3 / Bolt U3-B2: Config Renderer`.

## Checks Performed

- Mapping review against current proxy configuration model:
  - inbound listener fields align with `ListenerConfig` semantics (`bind_addr`, `banner_host`, policy mode, line-length guardrail).
  - outbound listener fields align with `OutboundListenerConfig` semantics (`bind_addr`, `banner_host`, global TLS policy, per-domain rules).
- Determinism review:
  - rendered JSON key ordering is stable,
  - per-domain rules are sorted canonically.
- Environment-profile review:
  - profile-selected bind addresses and banner host values are explicit in the artifact.
- Forward-compatibility review:
  - `require-pq` and `on_mismatch` are preserved in rendered domain rules for later runtime policy layers.
- CLI workflow review:
  - render validates first, then emits artifact to stdout/path with deterministic behavior.

## Evidence

- Test command:
  - `python -B -m unittest discover -s tests -v` (run in `verzola-control`)
- Result:
  - passed (`15` tests, `0` failures).
- Render-specific coverage:
  - snapshot-style output assertion,
  - deterministic cross-format rendering (`YAML` vs `TOML`),
  - environment-profile variance checks,
  - CLI render success/failure paths.

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-REL-03`:
  - deterministic render output supports reproducible config roll-forward/rollback diffing.
- `NFR-CMP-01`:
  - renderer artifacts and semantics are now documented and version-controlled.
- Risk register (`Policy misconfiguration by operators`):
  - render is gated by strict validation and emits predictable artifacts.

## Residual Risks

- Proxy runtime ingestion of `require-pq` enforcement remains in later units; renderer currently preserves these fields but runtime behavior is still pending.
- Environment profiles are static presets in this bolt; profile extension/governance is deferred.

## Sign-off

- Engineering compatibility review: complete for U3-B2 scope.
- Human compatibility sign-off: required before closing Unit U3.
