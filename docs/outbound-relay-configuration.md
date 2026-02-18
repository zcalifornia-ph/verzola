# Outbound Relay Configuration (Unit U2 / Bolt U2-B1)

## Purpose

This document covers outbound session orchestration introduced in `verzola-proxy/src/outbound/mod.rs` for Postfix relayhost mode.

## Reference Topology

```text
Postfix (relayhost=[127.0.0.1]:10025) -> VERZOLA outbound listener -> Remote MX
```

## Listener Configuration Example

```rust
use verzola_proxy::outbound::{OutboundListener, OutboundListenerConfig, NoopMxResolver};

let config = OutboundListenerConfig {
    bind_addr: "127.0.0.1:10025".parse().unwrap(),
    banner_host: "relay.verzola.test".to_string(),
    max_line_len: 4096,
};

let listener = OutboundListener::bind(config, NoopMxResolver)?;
```

Operational fields:

- `bind_addr`: Postfix-facing socket (`127.0.0.1:10025` in default relayhost wiring).
- `banner_host`: hostname advertised in outbound listener SMTP banner/replies.
- `max_line_len`: guardrail applied to command and DATA lines.

## Postfix Wiring

`main.cf`:

```ini
relayhost = [127.0.0.1]:10025
```

## Orchestration Behavior in This Bolt

- VERZOLA accepts SMTP from Postfix and stages `MAIL FROM` locally.
- On first `RCPT TO`, VERZOLA extracts recipient domain and resolves MX candidates.
- Candidates are attempted in deterministic order `(preference, exchange)`.
- For each candidate, VERZOLA validates remote SMTP readiness (`banner`, `EHLO`, `MAIL`) before relaying `RCPT/DATA`.
- Resolver/connection/bootstrap failures return `451 4.4.0` to preserve Postfix retries.

Current constraints (U2-B1 scope):

- One recipient domain per transaction.
- Advanced status-contract normalization is deferred to U2-B2.

## Validation Commands

```powershell
cd verzola-proxy
cargo test --test outbound_orchestration
```

Full suite:

```powershell
cd verzola-proxy
cargo test
```
