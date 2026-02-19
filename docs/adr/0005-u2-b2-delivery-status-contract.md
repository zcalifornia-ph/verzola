# ADR 0005: U2-B2 Delivery Status Contract (`250/4xx`)

## Status

Accepted for Bolt U2-B2 implementation.

## Context

Unit U2 requires deterministic Postfix-facing delivery semantics where success is acknowledged only after confirmed remote acceptance, and retry-safe outcomes preserve Postfix queue ownership (`NFR-REL-02`).

U2-B1 established outbound orchestration and MX failover but passed remote SMTP outcomes through directly. That behavior left outcome classes non-normalized and increased ambiguity around retry safety.

## Decision

- Add an explicit status-mapping layer in outbound relay handling for `RCPT`, `DATA` command, and final DATA payload completion.
- Normalize success responses:
  - `RCPT` accepted -> `250 2.1.5 Recipient accepted for remote delivery`
  - final DATA accepted -> `250 2.0.0 Message accepted by remote MX`
- Normalize retry-required outcomes to temporary defer:
  - remote `4xx` or `5xx` (or unexpected class) on delivery stages -> `451 4.4.0 ...` with deterministic stage/class/upstream metadata.
- Keep SMTP preflight behavior explicit:
  - only `3xx` on `DATA` command proceeds to payload streaming,
  - non-`3xx` preflight outcomes map to retry-safe `451`.
- Add contract tests covering transient and permanent remote outcomes to prove Postfix-facing defer behavior.

## Consequences

- Positive:
  - Postfix sees deterministic success/defer behavior aligned to Unit U2 acceptance intent.
  - Message safety is improved by preserving retry semantics across ambiguous or permanent upstream outcomes while policy layers are incomplete.
  - Contract behavior is test-covered and operator-visible through stable response patterns.
- Tradeoff:
  - Collapsing upstream `5xx` to `451` can increase retries for failures that may ultimately be permanent; this is an intentional safety-first posture for current project phase.
