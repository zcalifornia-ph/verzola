# Outbound Relay Configuration (Unit U2 / Bolts U2-B1, U2-B2, and U2-B3)

## Purpose

This document covers outbound relay behavior in `verzola-proxy/src/outbound/mod.rs` for Postfix relayhost mode, including orchestration (U2-B1), delivery status contract mapping (U2-B2), and outbound TLS policy application (U2-B3).

## Reference Topology

```text
Postfix (relayhost=[127.0.0.1]:10025) -> VERZOLA outbound listener -> Remote MX
```

## Listener Configuration Example

```rust
use verzola_proxy::outbound::{
    NoopMxResolver, OutboundDomainTlsPolicy, OutboundListener, OutboundListenerConfig,
    OutboundTlsPolicy,
};

let config = OutboundListenerConfig {
    bind_addr: "127.0.0.1:10025".parse().unwrap(),
    banner_host: "relay.verzola.test".to_string(),
    outbound_tls_policy: OutboundTlsPolicy::Opportunistic,
    per_domain_tls_policies: vec![
        OutboundDomainTlsPolicy::new("partner.example", OutboundTlsPolicy::RequireTls).unwrap(),
    ],
    max_line_len: 4096,
};

let listener = OutboundListener::bind(config, NoopMxResolver)?;
```

Operational fields:

- `bind_addr`: Postfix-facing socket (`127.0.0.1:10025` in default relayhost wiring).
- `banner_host`: hostname advertised in outbound listener SMTP banner/replies.
- `outbound_tls_policy`: global outbound policy (`opportunistic` or `require-tls`).
- `per_domain_tls_policies`: recipient-domain overrides that take precedence over the global policy.
- `max_line_len`: guardrail applied to command and DATA lines.

## Postfix Wiring

`main.cf`:

```ini
relayhost = [127.0.0.1]:10025
```

## Orchestration Behavior (U2-B1)

- VERZOLA accepts SMTP from Postfix and stages `MAIL FROM` locally.
- On first `RCPT TO`, VERZOLA extracts recipient domain and resolves MX candidates.
- Candidates are attempted in deterministic order `(preference, exchange)`.
- For each candidate, VERZOLA validates remote SMTP readiness (`banner`, `EHLO`, `MAIL`) before relaying `RCPT/DATA`.
- Resolver/connection/bootstrap failures return `451 4.4.0` to preserve Postfix retries.

Current constraint:

- One recipient domain per transaction.

## Delivery Status Contract (U2-B2)

Postfix-facing delivery outcomes are normalized to deterministic statuses:

- return `250` only after remote acceptance is confirmed,
- return retry-safe `4xx` (`451`) for remote refusal classes and relay/policy defer paths.

Status mapping matrix:

| Stage | Remote outcome | Postfix-facing reply |
|---|---|---|
| `RCPT TO` relay | `2xx` | `250 2.1.5 Recipient accepted for remote delivery` |
| `RCPT TO` relay | `4xx` or `5xx` (or unexpected non-`2xx`) | `451 4.4.0 Delivery deferred for retry (stage=rcpt, class=..., upstream=...)` |
| `DATA` command relay | `3xx` | `354 End data with <CR><LF>.<CR><LF>` |
| `DATA` command relay | non-`3xx` | `451 4.4.0 Delivery deferred for retry (stage=data-command, class=..., upstream=...)` |
| final `DATA` payload reply | `2xx` | `250 2.0.0 Message accepted by remote MX` |
| final `DATA` payload reply | `4xx` or `5xx` (or unexpected non-`2xx`) | `451 4.4.0 Delivery deferred for retry (stage=data-final, class=..., upstream=...)` |

Operator expectations:

- Postfix should treat all `451` responses as defer/retry and retain queue ownership.
- Delivery is considered complete only on final `250` after payload relay.
- Upstream status classes are intentionally collapsed into retry-safe defer semantics to reduce message-loss risk while policy layering is still in progress.

Troubleshooting matrix:

| Symptom in Postfix logs | Likely cause | Verification path |
|---|---|---|
| `451 ... stage=rcpt, class=remote-transient` | remote MX transient issue (`4xx`) | verify remote MX health and DNS/network reachability |
| `451 ... stage=rcpt, class=remote-permanent` | remote MX permanent refusal (`5xx`), intentionally deferred for safety | inspect recipient/domain policy, remote acceptance rules, and queue retry trend |
| `451 ... stage=data-command` | remote MX rejected `DATA` preflight | inspect remote SMTP capability/policy and command transcript |
| `451 ... stage=data-final` | remote MX rejected payload after DATA transfer | inspect content/policy rejection reason and message trace evidence |
| `451 4.4.0 Outbound MX temporarily unavailable` | resolver/connect/bootstrap failure before RCPT acceptance | verify MX records and remote socket availability |

## Outbound TLS Policy Application (U2-B3)

Policy evaluation order:

1. determine recipient domain from first accepted `RCPT TO`;
2. apply per-domain override if present;
3. otherwise apply global `outbound_tls_policy`.

Supported policy modes:

- `opportunistic`:
  - if STARTTLS is available and succeeds, session proceeds with negotiated TLS;
  - if STARTTLS is unavailable or fails, relay falls back to plaintext and continues.
- `require-tls`:
  - if STARTTLS is unavailable or fails, relay defers with `451 4.7.5 Outbound TLS policy defer: ...`;
  - Postfix retains queue ownership and retries according to its schedule.

Downgrade/defer matrix:

| Effective policy | Remote capability/outcome | Postfix-facing result |
|---|---|---|
| `opportunistic` | no STARTTLS advertised | continue over plaintext (`RCPT` proceeds) |
| `opportunistic` | STARTTLS advertised, handshake path fails | fallback to plaintext and continue |
| `require-tls` | no STARTTLS advertised | `451 4.7.5 Outbound TLS policy defer: ...` |
| `require-tls` | STARTTLS advertised but non-`2xx` STARTTLS/EHLO-after-STARTTLS | `451 4.7.5 Outbound TLS policy defer: ...` |

Policy override examples:

- Global opportunistic, strict partner:
  - `outbound_tls_policy = opportunistic`
  - `per_domain_tls_policies["partner.example"] = require-tls`
- Global require-tls, compatibility carve-out:
  - `outbound_tls_policy = require-tls`
  - `per_domain_tls_policies["legacy.example"] = opportunistic`

## Validation Commands

```powershell
cd verzola-proxy
cargo test --test outbound_orchestration
cargo test --test outbound_status_contract
cargo test --test outbound_tls_policy
```

Full suite:

```powershell
cd verzola-proxy
cargo test
```
