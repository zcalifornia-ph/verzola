# Inbound Postfix Integration Notes (Unit U1 / Bolt U1-B2)

## Purpose

This document describes how to wire VERZOLA inbound relay mode to a local Postfix instance so SMTP commands and DATA are streamed to loopback Postfix.

## Reference Topology

```text
Internet SMTP client -> VERZOLA inbound listener -> Postfix loopback listener
```

Example local wiring:

- VERZOLA inbound bind: `0.0.0.0:25` (or `:587` when submission mode is enabled)
- VERZOLA relay target: `127.0.0.1:2525`
- Postfix listener: `127.0.0.1:2525`

## Postfix Configuration Snippets

`main.cf` (limit public exposure, keep SMTP service loopback-only for the sidecar handoff):

```ini
inet_interfaces = loopback-only
```

`master.cf` (dedicated loopback listener for VERZOLA handoff):

```ini
2525      inet  n       -       n       -       -       smtpd
```

## Relay Behavior Notes

- VERZOLA accepts client-side STARTTLS and SMTP command flow.
- On first relay-required command (`MAIL`/`RCPT`/`DATA`), VERZOLA opens a loopback Postfix session and sends upstream `EHLO`.
- DATA content is relayed line-by-line (bounded by `max_line_len`) to avoid full-message buffering.
- If loopback relay becomes unavailable, VERZOLA returns temporary failure (`451`) to preserve retry behavior.

## Validation Checklist

- Verify Postfix is listening on `127.0.0.1:2525`.
- Verify VERZOLA is configured with `postfix_upstream_addr = 127.0.0.1:2525`.
- Run integration tests:

```powershell
cd verzola-proxy
cargo test --test inbound_forwarder
```

- Confirm both large-message and concurrent-session tests pass.
