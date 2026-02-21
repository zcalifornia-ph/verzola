# ADR 0007: U3-B1 Schema and Validation Engine

## Status

Accepted for Bolt U3-B1 implementation.

## Context

Unit U3 requires a control-plane surface (`verzolactl`) that can validate policy-as-code inputs before they affect proxy behavior. The repository previously had no control-plane package, schema model, or validation diagnostics for malformed policy files.

U3-B1 focuses on the first deployable slice:

- schema model,
- parser support for policy files (`YAML`/`TOML`),
- strict validation mode with actionable diagnostics.

## Decision

- Create a new Python package at `verzola-control/` as the control-plane baseline.
- Implement schema types in `verzola_control.policy.model` for:
  - listener policies,
  - domain overrides,
  - DNS capability hints.
- Implement parser support in `verzola_control.policy.parser`:
  - `TOML` via `tomllib`,
  - constrained YAML mapping parser for the project policy format.
- Implement strict validation engine in `verzola_control.validate.engine`:
  - rejects unknown fields when strict mode is enabled,
  - validates policy mode values (`opportunistic`, `require-tls`, `require-pq`),
  - validates domain keys and duplicate normalized domain rules,
  - emits diagnostics with file path, field path, message, and suggestion.
- Add `verzolactl validate` CLI command in `verzola_control.cli` to expose validation behavior.

## Consequences

- Positive:
  - control-plane policy validation now exists as a real executable component.
  - malformed configs fail fast with actionable diagnostics.
  - strict mode enforces schema hygiene and prevents silent drift.
- Tradeoff:
  - YAML support is intentionally limited to project policy structure and does not implement full YAML features (lists/anchors/tags) in this bolt.
