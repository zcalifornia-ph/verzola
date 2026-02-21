# ADR 0008: U3-B2 Config Renderer

## Status

Accepted for Bolt U3-B2 implementation.

## Context

Unit U3 requires deterministic policy rendering so validated policy files can produce reviewable, reproducible proxy-facing configuration artifacts.  
U3-B1 delivered schema validation and diagnostics, but there was no renderer workflow, no environment profile abstraction, and no generated artifact contract.

## Decision

- Add renderer module at `verzola-control/verzola_control/render/engine.py`.
- Implement intermediate representation (IR) dataclasses for:
  - environment profile,
  - inbound listener config,
  - outbound listener config,
  - per-domain outbound policy rules,
  - top-level rendered artifact.
- Render effective policy as deterministic JSON:
  - stable key ordering (`sort_keys=True`),
  - canonical domain-rule ordering (sorted domain keys),
  - explicit environment marker in the artifact.
- Add environment profiles (`dev`, `staging`, `prod`) to make rendered output environment-aware without changing policy source files.
- Extend CLI with `verzolactl render`:
  - input policy path,
  - environment selector,
  - output path or stdout.
- Preserve strict validation behavior before rendering so malformed policy never produces a render artifact.

## Consequences

- Positive:
  - policy-to-artifact rendering now exists as an executable and test-covered workflow.
  - rendered outputs are deterministic for snapshot testing and change-review diffing.
  - environment-specific deployment settings are explicit and reproducible.
- Tradeoff:
  - current artifact schema preserves `require-pq` rules for forward compatibility, but full runtime enforcement remains in later units (`U4`+).
