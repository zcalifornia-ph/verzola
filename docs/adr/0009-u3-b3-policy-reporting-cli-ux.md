# ADR 0009: U3-B3 Policy Reporting and CLI UX

## Status

Accepted for Bolt U3-B3 implementation.

## Context

Unit U3 needed one remaining capability after validation and rendering: policy reports that summarize domain posture and explicitly surface actionable gaps.  
Without this, operators could validate syntax and render config, but still lacked a structured risk/readiness view in the CLI workflow.

## Decision

- Add reporting module at `verzola-control/verzola_control/report/engine.py`.
- Introduce deterministic report model with:
  - posture summary,
  - sorted domain override detail,
  - detected gaps (code, severity, scope, recommendation).
- Define report severities:
  - `critical`,
  - `warning`,
  - `info`.
- Extend CLI with `verzolactl report` and options:
  - `--environment`,
  - `--format (text|json)`,
  - `--output`,
  - `--no-strict`.
- Keep report generation validation-gated for strict schema safety parity with existing CLI flows.

## Consequences

- Positive:
  - Unit U3 acceptance criteria for policy posture/gap reporting is now executable and test-covered.
  - Operators have one CLI workflow for validate -> render -> report.
  - JSON report output provides structured artifact evidence for CI and change reviews.
- Tradeoff:
  - Gap detection rules are intentionally heuristic and conservative for this bolt; deeper runtime/security semantics remain a future extension.

