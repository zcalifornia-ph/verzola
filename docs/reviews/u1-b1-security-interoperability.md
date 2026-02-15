# U1-B1 Security and Interoperability Review

## Scope

Artifact review for `Listener and STARTTLS Negotiation` (`Unit U1`, `Bolt U1-B1`).

## Security Checks

- Input bounds:
  - command/data line length guardrail with configurable `max_line_len`.
- Protocol safety:
  - `STARTTLS` requires prior `EHLO`.
  - envelope commands require `EHLO` (and require re-`EHLO` after STARTTLS).
- Failure mapping:
  - temporary TLS handshake failures mapped to SMTP `454 4.7.0`.
- Secret handling:
  - no certificate paths hard-coded in source; production cert loading remains adapter responsibility.

## Interoperability Checks

- SMTP replies use standard status classes (`220`, `250`, `354`, `454`, `5xx` protocol errors).
- Multiline `EHLO` capability format uses `250-...` continuation and final `250 ...`.
- STARTTLS advertisement suppressed after TLS is marked active.

## Open Items

- Plug in production TLS adapter (`TlsUpgrader`) with certificate loading and policy alignment.
- Run interoperability tests against real SMTP clients (Postfix, OpenSMTPD, swaks) after adapter integration.

## Sign-off

- Engineering pre-review: complete.
- Human security/interoperability sign-off: required before closing Unit U1.
