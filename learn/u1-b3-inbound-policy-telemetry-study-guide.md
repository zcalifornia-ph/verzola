# Unit U1-B3 Study Guide

## Title
Inbound SMTP Policy Enforcement + Session Telemetry (Rust, VERZOLA)

## Quick Diagnostic Read

You are ready for this guide if you can:

- read Rust enums/structs and `match` logic,
- follow SMTP command ordering (`EHLO`, `STARTTLS`, `MAIL`, `RCPT`, `DATA`),
- understand why tests are used as behavior contracts.

What is new and high-value in this bolt:

- policy-mode design (`opportunistic` vs `require-tls`),
- deterministic rejection mapping (`530 5.7.0 Must issue STARTTLS first`),
- adding telemetry fields that are easy to assert in tests,
- fail-fast config validation for unsafe policy combinations.

## One-Sentence Objective

Understand how U1-B3 enforces inbound TLS policy deterministically and records per-session telemetry that proves policy outcomes.

## Why This Bolt Matters

U1-B1 built protocol state discipline. U1-B2 built relay streaming. U1-B3 adds the policy gate that decides whether plaintext envelope/data commands are allowed.

If this bolt is wrong, operators get ambiguous behavior:

- clients may send plaintext when policy expects TLS,
- SMTP responses become inconsistent,
- observability data is too weak for troubleshooting.

U1-B3 is also a prerequisite for stronger policy work later (for example strict PQ handling in Unit U4).

## Plan A / Plan B

### Plan A (Recommended): Test-First Walkthrough (90-140 minutes)

1. Read `verzola-proxy/tests/inbound_policy_telemetry.rs` top to bottom.
2. Run `cargo test --test inbound_policy_telemetry`.
3. Map each test to the exact branch in `verzola-proxy/src/inbound/mod.rs`.
4. Do one drill from this guide.

### Plan B: Policy-First Walkthrough (70-120 minutes)

1. Read `docs/inbound-policy-telemetry.md`.
2. Read ADR `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`.
3. Use tests to verify your understanding.

Use Plan B if the implementation file feels too dense at first.

## System View (Mental Model)

```text
Inbound session starts
  -> EHLO required first (existing SMTP ordering)
  -> Policy check on MAIL/RCPT/DATA:
      opportunistic:
        allow plaintext path after EHLO
      require-tls:
        if TLS inactive -> 530 5.7.0
        if TLS active   -> allow
  -> STARTTLS path:
      attempt upgrade
      success -> tls_active=true, EHLO required again
      failure -> 454 temporary TLS unavailable
  -> Session summary captures policy + telemetry counters
```

Think of U1-B3 as adding one policy decision layer on top of the existing state machine, not replacing it.

## What We Built (Artifact Map)

- Core implementation:
  - `verzola-proxy/src/inbound/mod.rs`
- Configuration usage updates:
  - `verzola-proxy/src/main.rs`
  - `verzola-proxy/tests/inbound_starttls.rs`
  - `verzola-proxy/tests/inbound_forwarder.rs`
- New policy/telemetry tests:
  - `verzola-proxy/tests/inbound_policy_telemetry.rs`
- Design and operations artifacts:
  - `docs/inbound-policy-telemetry.md`
  - `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`
  - `docs/reviews/u1-b3-operational-readiness.md`
  - `docs/bolts/u1-b3-traceability.md`
- Requirements evidence:
  - `REQUIREMENTS.md` (U1-B3 checked with test evidence)

## Guided Walkthrough (Implementation)

## 1) New Policy Enum and Config Field

Added `InboundTlsPolicy`:

- `Opportunistic`
- `RequireTls`

Added to `ListenerConfig` as `inbound_tls_policy`.

Why this is good design:

- explicit policy mode in config,
- no hidden policy behavior inside command handling,
- easy to test each mode.

## 2) Validation Guardrail for Unsafe Config

In config validation:

- `require-tls` with `advertise_starttls=false` is rejected at startup.

Why this matters:

- avoids impossible runtime behavior (requiring TLS while not advertising upgrade path),
- fails early with an actionable error.

## 3) Policy Decision Point in MAIL/RCPT/DATA

`can_process_mail_command(...)` now returns:

- EHLO sequencing error when ordering is wrong,
- TLS-required error when policy says TLS is mandatory but session is still plaintext.

Command behavior:

- `MAIL`, `RCPT`, and `DATA` return `530 5.7.0 Must issue STARTTLS first` when policy requires TLS and `tls_active` is false.

This gives a clear and deterministic contract for clients.

## 4) Session Telemetry Schema

`SessionSummary` now includes:

- `inbound_tls_policy`
- `telemetry.starttls_offered`
- `telemetry.starttls_attempts`
- `telemetry.tls_upgrade_failures`
- `telemetry.require_tls_rejections`
- `telemetry.relay_temporary_failures`

Why session-scoped telemetry first:

- low complexity,
- strong testability,
- easy evolution toward aggregate metrics in Unit U5.

## 5) No Regression of Existing Behavior

The bolt keeps existing semantics intact:

- STARTTLS failure still maps to `454`,
- relay failures still map to temporary `451`,
- EHLO sequencing rules still apply.

U1-B3 extends behavior without breaking U1-B1/U1-B2 contracts.

## Guided Walkthrough (Tests as Specs)

Use these tests as your primary learning path:

1. `opportunistic_policy_allows_plaintext_mail_flow`
   - proves plaintext path is allowed after EHLO in opportunistic mode.

2. `require_tls_policy_rejects_plaintext_until_starttls`
   - proves `MAIL/RCPT/DATA` each reject with `530` until TLS is active.
   - proves flow works after STARTTLS + re-EHLO.

3. `require_tls_listener_config_requires_starttls_advertisement`
   - proves invalid config is blocked at bind-time.

4. `telemetry_tracks_tls_failures_and_policy_rejections`
   - proves counters increment on STARTTLS failure and policy rejection path.

If you can explain each assertion in these tests, you understand this bolt.

## Copy-Paste Commands

From repo root:

```powershell
cd d:\Programming\Repositories\verzola\verzola-proxy
cargo test --test inbound_policy_telemetry
```

Full inbound coverage:

```powershell
cd d:\Programming\Repositories\verzola\verzola-proxy
cargo test
```

Expected focus result:

- 4 passing tests in `inbound_policy_telemetry.rs`.

## Pitfalls + Debugging (High Yield)

### 1) Policy logic seems ignored

Check:

- is `inbound_tls_policy` explicitly set in your test config?
- are you sending `EHLO` before testing `MAIL/RCPT/DATA`?

### 2) `require-tls` test fails unexpectedly

Check:

- did STARTTLS actually succeed in your scenario?
- after successful STARTTLS, did you send EHLO again?

### 3) Confusing error codes (`503` vs `530`)

Rule:

- `503` = command sequencing issue (EHLO state problem),
- `530` = policy requirement not met (TLS required).

### 4) Telemetry assertions flaky

Check:

- assert final `SessionSummary` after joining server thread,
- avoid asserting counters before session teardown.

## Skill Transfer: What You Should Internalize

After this bolt, you should be able to explain:

1. The difference between protocol-order errors and policy-enforcement errors.
2. Why config validation should block impossible policy combinations.
3. Why deterministic SMTP mappings improve reliability and operator trust.
4. How to design telemetry that is minimal but testable.

## Practice Drill (25-50 Minutes)

### Task

Add one test for `require-tls` where:

- client sends EHLO,
- tries `DATA` immediately (without MAIL/RCPT and without TLS),
- assert `530 5.7.0 Must issue STARTTLS first`,
- then send `STARTTLS` + EHLO and verify DATA still follows normal SMTP sequencing afterward.

### Self-Check

You pass if:

- your test fails first,
- implementation remains unchanged (or minimally changed) to make it pass,
- all inbound tests remain green.

## Mini Competency Map (For This Topic)

- Level 1: explain the two policy modes and expected SMTP replies.
- Level 2: write and debug a new policy-matrix test.
- Level 3: extend telemetry schema without breaking existing tests.
- Level 4: review policy code for regressions and ambiguous mappings.

## 24-72 Hour Next Steps

1. Do the drill above.
2. Trace where this policy layer will integrate with outbound U2 flow concepts.
3. Sketch how per-session telemetry can roll up into Unit U5 metrics.
4. Keep practicing test-first changes on protocol logic before adding new features.

---

This guide is intentionally execution-first: run tests, map decisions, then modify one small behavior safely.
