# Unit U1-B1 Study Guide

## Title
Inbound SMTP Listener + STARTTLS Negotiation (Rust, VERZOLA)

## Quick Diagnostic Read

You are likely ready for this if you can:

- read basic Rust control flow (`match`, structs, traits),
- follow socket I/O in a loop,
- run tests and reason from pass/fail behavior.

What is new (and high-value) in this bolt:

- protocol state machines,
- SMTP response-code discipline (`220`, `250`, `454`, `503`),
- designing for later TLS integration without blocking current progress.

## One-Sentence Objective

Understand exactly how U1-B1 turns a raw TCP SMTP session into a deterministic, test-driven STARTTLS flow that is secure, debuggable, and ready for future TLS adapter wiring.

## Why This Bolt Matters

Unit U1 is the inbound SMTP proxy. Bolt U1-B1 is the foundation because:

- every inbound session starts at this listener,
- protocol errors must be predictable,
- STARTTLS negotiation behavior must be explicit before streaming/relay work in later bolts.

If this layer is unclear, U1-B2 and U1-B3 become fragile.

## Plan A / Plan B

### Plan A (Recommended): Code-First in 90-120 Minutes

1. Read `verzola-proxy/src/inbound/mod.rs` top to bottom once.
2. Run tests in `verzola-proxy/tests/inbound_starttls.rs`.
3. Map each test to the exact server logic branch.
4. Do one mini-exercise at the end of this guide.

### Plan B: Protocol-First in 60-90 Minutes

1. Learn SMTP/STARTTLS flow from the state diagram below.
2. Read tests first (`inbound_starttls.rs`) as executable behavior specs.
3. Read implementation second (`inbound/mod.rs`) to confirm each rule.

Use Plan B if you feel overloaded by raw implementation details.

## System View (Mental Model)

```
TCP connect
  -> 220 banner
  -> EHLO/HELO
      -> 250 capabilities (+ STARTTLS when allowed and not yet active)
  -> STARTTLS
      -> 220 Ready to start TLS
      -> TLS upgrader:
          success  -> tls_active=true, EHLO required again
          failure  -> 454 temporary TLS unavailable
  -> MAIL/RCPT/DATA allowed only after required EHLO state
  -> QUIT -> 221
```

Think of this as a finite-state machine with two key booleans:

- `ehlo_seen`
- `tls_active`

That is enough state for this bolt to enforce correct command ordering.

## What We Built (Artifact Map)

- Core listener and protocol logic:
  - `verzola-proxy/src/inbound/mod.rs`
- Crate entry points:
  - `verzola-proxy/src/lib.rs`
  - `verzola-proxy/src/main.rs`
- Behavior tests:
  - `verzola-proxy/tests/inbound_starttls.rs`
- Design and operations docs:
  - `docs/inbound-listener.md`
  - `docs/adr/0001-u1-b1-listener-starttls-state-machine.md`
  - `docs/reviews/u1-b1-security-interoperability.md`
  - `docs/bolts/u1-b1-traceability.md`
- Requirements proof record:
  - `REQUIREMENTS.md` (U1-B1 checked + evidence)

## Guided Walkthrough (Implementation)

## 1) Config and Guardrails

In `ListenerConfig`, four fields define operational behavior:

- bind location,
- banner hostname,
- STARTTLS advertisement toggle,
- max line length.

Validation enforces:

- non-empty `banner_host`,
- `max_line_len >= 512`.

Why this matters:

- prevents invalid startup configuration,
- gives deterministic safety bounds against malformed input.

## 2) TLS Decoupling Through a Trait

`TlsUpgrader` is an interface:

- protocol layer calls `upgrade(&mut TcpStream)`,
- concrete TLS implementation can be plugged later,
- tests can inject success/failure behavior immediately.

Current adapters:

- `NoopTlsUpgrader`: always succeeds (useful for protocol tests),
- `FailingTlsUpgrader` in tests: simulates temporary TLS failure.

This is a strong engineering decision for staged delivery.

## 3) Session State and Command Loop

`SessionState` tracks:

- `tls_active`,
- `ehlo_seen`,
- counters (`command_count`, `protocol_errors`).

Flow highlights:

- on connect -> send `220`,
- on `EHLO`/`HELO` -> set `ehlo_seen=true`, advertise capabilities,
- on `STARTTLS`:
  - reject if unsupported (`502`),
  - reject if already active (`503`),
  - reject if no EHLO yet (`503`),
  - otherwise send `220 Ready to start TLS` and run upgrader,
  - map temporary failure to `454`.

Crucial rule:

- after successful STARTTLS, EHLO is required again before `MAIL/RCPT/DATA`.

## 4) SMTP Reply Discipline

Code chooses specific statuses by situation:

- `220`: banner and TLS readiness,
- `250`: success responses (EHLO capability list, MAIL/RCPT success),
- `354`: DATA body prompt,
- `454`: temporary TLS issue (retry-safe behavior),
- `503`: bad command sequence,
- `502`: unsupported command/capability branch.

This is not cosmetic. Status-code discipline determines client behavior and reliability semantics.

## 5) Data Handling

`DATA` mode reads lines until `.` terminator, enforcing line-length limits.

Even though U1-B2 covers full streaming relay, this bolt already sets safe parser boundaries and failure handling.

## Guided Walkthrough (Tests as Specs)

The tests in `verzola-proxy/tests/inbound_starttls.rs` are the fastest path to understanding behavior:

1. `starttls_success_requires_ehlo_reset`
   - verifies STARTTLS is advertised pre-upgrade,
   - verifies `MAIL` is rejected until EHLO is sent again,
   - verifies STARTTLS is no longer advertised after TLS active.

2. `starttls_failure_maps_to_454`
   - injects a failing upgrader,
   - verifies temporary handshake failure maps to `454`.

3. `starttls_before_ehlo_is_rejected`
   - verifies sequence enforcement (`503`).

Takeaway:

- tests are not only validation; they are executable protocol documentation.

## Copy-Paste Commands

### Standard (if `cargo` is on PATH)

```powershell
cd d:\Programming\Repositories\verzola\verzola-proxy
cargo test
```

### Windows + MSVC toolchain setup path (what we used)

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && cd /d d:\Programming\Repositories\verzola\verzola-proxy && cargo test'
```

Expected result:

- 3 integration tests pass in `inbound_starttls.rs`.

## Pitfalls + Debugging (High Yield)

### 1) `cargo` not found

- Cause: Rust not installed or PATH not loaded in current shell.
- Fix: install via `rustup`, then reopen terminal or prepend `%USERPROFILE%\.cargo\bin` in command.

### 2) `link.exe` not found

- Cause: Rust MSVC target without Visual Studio Build Tools.
- Fix: install VS Build Tools with C++ workload.

### 3) `Access is denied` for `target`

- Cause: environment/sandbox/permission mismatch.
- Fix: verify folder ACLs and run in a shell with proper permissions.

### 4) STARTTLS behavior looks inconsistent

- Use tests first. If a behavior is unclear, codify expectation in a failing test, then adjust implementation.

## Skill Transfer: What You Should Internalize

After this bolt, you should be able to explain:

1. Why a protocol parser needs explicit state.
2. Why temporary TLS errors should map to retry-safe statuses.
3. Why traits/interfaces let you deliver incrementally without fake architecture.
4. Why tests are protocol contracts, not an afterthought.

If you can explain those four, you actually understood the work.

## Practice Drill (20-40 Minutes)

### Task

Add one test that validates behavior when STARTTLS advertising is disabled.

Hint:

- set `advertise_starttls=false` in test listener config,
- send `EHLO`,
- assert `250-STARTTLS` is absent,
- send `STARTTLS`,
- assert `502 5.5.1 STARTTLS not supported`.

### Self-Check

You pass this drill if:

- test fails first,
- implementation satisfies behavior without breaking existing 3 tests,
- all tests pass after your change.

## Mini Competency Map (for This Topic)

- Level 1: Can run tests and explain each assertion in plain English.
- Level 2: Can modify protocol rules and update tests correctly.
- Level 3: Can plug in a real TLS adapter while preserving command-state semantics.
- Level 4: Can review another implementation for protocol/security regressions.

## 24-72 Hour Next Steps

1. Do the STARTTLS-disabled drill.
2. Add one more test for line-length enforcement.
3. Start U1-B2 by sketching command/data forwarding boundaries and backpressure handling.
4. Keep one short engineering note per bolt: decision, tradeoff, evidence.

---

This guide is intentionally practical: read code, run tests, map behavior, then modify one thing safely.
