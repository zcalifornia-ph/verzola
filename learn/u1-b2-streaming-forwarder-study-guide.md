# Unit U1-B2 Study Guide

## Title
Inbound SMTP Streaming Forwarder to Postfix Loopback (Rust, VERZOLA)

## Quick Diagnostic Read

You are likely ready for this guide if you can:

- read Rust structs and `match` branches comfortably,
- follow socket read/write loops without panic,
- understand basic SMTP flow: `EHLO`, `MAIL`, `RCPT`, `DATA`, `QUIT`.

What is new and high-value in this bolt:

- building an SMTP-to-SMTP relay bridge,
- streaming DATA safely without full-message buffering,
- handling temporary failures so retries stay safe,
- validating concurrency behavior with integration tests.

## One-Sentence Objective

Understand how U1-B2 turns the inbound listener into a Postfix loopback forwarder that streams commands and message data safely, while staying deterministic under large payloads and concurrent sessions.

## Why This Bolt Matters

U1-B1 gave you inbound STARTTLS/session rules. U1-B2 makes that path useful in real deployments by forwarding accepted SMTP traffic to loopback Postfix (`localhost:2525` style topology).

If U1-B2 is wrong, the sidecar can:

- buffer too much memory,
- break SMTP ordering,
- lose retry safety during relay errors.

This bolt is the foundation before policy and telemetry work in U1-B3.

## Plan A / Plan B

### Plan A (Recommended): Tests First, Then Implementation (90-150 min)

1. Read `verzola-proxy/tests/inbound_forwarder.rs`.
2. Run `cargo test --test inbound_forwarder`.
3. Walk through `verzola-proxy/src/inbound/mod.rs` and map each test step to code branches.

### Plan B: Architecture First, Then Tests (70-120 min)

1. Read this guide's relay data-flow diagram.
2. Read `docs/adr/0002-u1-b2-streaming-forwarder.md`.
3. Use tests to verify your understanding.

Use Plan B if code-first feels too dense.

## Mental Model (Relay Data Flow)

```text
Internet SMTP client
  -> VERZOLA inbound listener session
    -> (lazy) connect to loopback Postfix
      -> read upstream 220 banner
      -> send upstream EHLO
    -> relay MAIL / RCPT / DATA commands upstream
    -> for DATA body:
         read one line from client
         enforce max_line_len
         write same line upstream
         flush
       until "." terminator
    -> send upstream final reply back to client
```

Core idea: this is a streaming pipe, not a message store.

## What We Built (Artifact Map)

- Relay implementation:
  - `verzola-proxy/src/inbound/mod.rs`
- Relay tests:
  - `verzola-proxy/tests/inbound_forwarder.rs`
- Existing STARTTLS tests kept passing:
  - `verzola-proxy/tests/inbound_starttls.rs`
- Design and traceability:
  - `docs/adr/0002-u1-b2-streaming-forwarder.md`
  - `docs/inbound-postfix-integration.md`
  - `docs/reviews/u1-b2-performance-review.md`
  - `docs/bolts/u1-b2-traceability.md`
- Completion evidence:
  - `REQUIREMENTS.md` (Bolt U1-B2 checked)

## Guided Walkthrough (Implementation)

## 1) Config Extension for Relay Mode

`ListenerConfig` gained:

- `postfix_upstream_addr: Option<SocketAddr>`

Validation now also checks:

- upstream address must not equal bind address.

Why this is important:

- avoids accidental self-loop relay,
- keeps non-relay mode possible (`None`) for isolated listener behavior.

## 2) Lazy Postfix Session Bootstrap

`PostfixRelay::connect(...)` performs:

1. TCP connect to upstream Postfix.
2. Read upstream banner (`220` expected class).
3. Send upstream `EHLO`.
4. Require upstream 2xx `EHLO` response.

Why lazy connection:

- only pay relay connection cost when needed (`MAIL/RCPT/DATA` path),
- keeps idle/no-envelope sessions lightweight.

## 3) Command Relay Path

For `MAIL`, `RCPT`, `RSET`, `NOOP`, and `QUIT` in relay mode:

- forward command upstream,
- parse upstream SMTP reply (including multiline format),
- mirror reply back to the client.

Safety behavior:

- if relay operation fails, return `451` temporary failure and reset relay state for retry-safe behavior.

## 4) DATA Streaming + Backpressure

For `DATA` in relay mode:

1. Relay `DATA` command upstream.
2. If upstream gives 3xx (typically `354`), start streaming body.
3. Read client data one line at a time.
4. Enforce `max_line_len` per line.
5. Write each line upstream and flush.
6. Stop at `.` terminator.
7. Read upstream final accept/defer reply and return it to client.

Why this design is high-quality:

- no full message buffering in process memory,
- natural backpressure from blocking socket I/O,
- straightforward error handling at each stage.

## 5) STARTTLS Interaction and Relay Reset

After successful STARTTLS upgrade:

- `ehlo_seen` resets (expected SMTP behavior),
- relay handle is dropped (`relay = None`) so upstream session is not reused across protocol phase change.

This keeps session state clean and predictable.

## 6) Concurrency Support for Tests

`InboundListener::serve_n(session_count)` was added to accept multiple sessions and process each in a worker thread.

Why this matters:

- gives deterministic integration coverage for concurrent relay behavior,
- verifies no cross-talk between sessions.

## Guided Walkthrough (Tests as Executable Specs)

## Test 1: `relays_large_data_block_to_postfix_loopback`

Proves:

- envelope and DATA flow are relayed end-to-end,
- large payload (~614 KB) is accepted and forwarded,
- reply mapping remains correct.

## Test 2: `relays_concurrent_sessions_without_cross_talk`

Proves:

- two clients can relay simultaneously,
- each session completes without protocol errors,
- aggregate bytes and message counts match expectations.

Key test technique:

- a mock Postfix server is implemented in-test using `TcpListener`, so behavior is deterministic without external dependencies.

## Copy-Paste Commands

From repo root:

```powershell
cd d:\Programming\Repositories\verzola\verzola-proxy
cargo test --test inbound_forwarder
```

Full inbound suite:

```powershell
cd d:\Programming\Repositories\verzola\verzola-proxy
cargo test
```

Expected for this bolt:

- `relays_large_data_block_to_postfix_loopback ... ok`
- `relays_concurrent_sessions_without_cross_talk ... ok`

## Pitfalls + Debugging (High Yield)

### 1) Relay connection fails immediately

Symptoms:

- `451 4.4.0 Postfix relay unavailable: ...`

Check:

- is Postfix (or mock upstream) listening on configured loopback address?
- did you accidentally set upstream equal to bind address?

### 2) DATA relay fails mid-message

Symptoms:

- `451 4.3.0 DATA relay failure: ...`

Check:

- upstream connection dropped during DATA,
- line length exceeded `max_line_len`,
- client closed connection before terminator.

### 3) Multiline upstream replies break parsing

Symptoms:

- invalid SMTP reply parsing error.

Check:

- upstream replies must follow SMTP reply format (`250-...`, final `250 ...` with same code).

### 4) Concurrent test flakes

Check:

- barrier synchronization counts,
- socket timeouts too short for your machine.

## Mini Practice Drill (30-60 Minutes)

### Drill A (Recommended)

Add a test where upstream Postfix returns `450` on `RCPT` and assert client receives that exact response.

Learning target:

- verify reply propagation fidelity in relay mode.

### Drill B

Add a test where one DATA line exceeds `max_line_len` and assert temporary failure mapping behavior.

Learning target:

- reinforce bounded-streaming safety logic.

### Self-Check

You understood U1-B2 if you can explain:

1. Why lazy upstream connect is used.
2. How line-by-line DATA forwarding avoids memory spikes.
3. Why relay errors are mapped to temporary failures.
4. How concurrency is validated without a real Postfix daemon.

## Competency Map (For This Topic)

- Level 1: read tests and explain the relay flow.
- Level 2: modify relay behavior and keep tests green.
- Level 3: add new failure-mode tests confidently.
- Level 4: review SMTP relay code for correctness and retry safety.

## 24-72 Hour Next Steps

1. Do Drill A and Drill B.
2. Sketch where U1-B3 policy checks should hook into `MAIL/RCPT/DATA` relay flow.
3. Add a short note comparing `451` usage in U1-B2 vs future stricter policy behaviors.
4. Keep practicing test-first changes on protocol code to reduce regression risk.

---

This guide is designed for fast skill transfer: model the flow, map it to tests, then ship one safe behavior change.
