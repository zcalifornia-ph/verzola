# U2-B1 Protocol Behavior Review (Outbound Session Orchestration)

## Scope

Review artifact for `Unit U2 / Bolt U2-B1: Outbound Session Orchestration`.

## Checks Performed

- Postfix-facing protocol sequencing:
  - enforces `EHLO` before `MAIL`,
  - enforces `MAIL` before `RCPT`,
  - enforces `RCPT` before `DATA`.
- MX routing strategy:
  - recipient domain extracted from `RCPT TO`,
  - candidates sorted by `(preference, exchange)` and attempted in order.
- Remote SMTP bootstrap:
  - candidate accepted only if remote banner, `EHLO`, and staged `MAIL FROM` are all `2xx`.
- Failure safety:
  - resolver/connect/bootstrap failures map to temporary `451 4.4.0` responses.
- Data-plane behavior:
  - DATA payload relayed line-by-line with `max_line_len` enforcement.

## Evidence

- Test command:
  - `cargo test --test outbound_orchestration`
- Passing tests:
  - `orchestrates_outbound_delivery_with_mx_failover`
  - `returns_temporary_failure_when_all_mx_candidates_are_unavailable`

## NFR/Risk Mapping (Current Bolt Scope)

- `NFR-REL-02`:
  - temporary error paths use `451` so Postfix retry semantics remain intact.
- Unit U2 acceptance scaffolding:
  - remote session orchestration path is now implemented and test-covered.

## Residual Risks

- Production MX resolution backend is not yet wired (trait seam exists, tests use static resolver).
- Mixed-domain recipient transactions are intentionally constrained in this bolt.
- Detailed `250/4xx` outcome contract normalization is tracked in U2-B2.

## Sign-off

- Engineering pre-review: complete for U2-B1 scope.
- Human protocol behavior sign-off: required before closing Unit U2.
