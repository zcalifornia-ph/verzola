# Bolt U1-B1 Traceability

## Contract Extract (from `REQUIREMENTS.md`)

- Goal: Listener and STARTTLS negotiation for inbound SMTP.
- Subtasks:
  - Design: listener configuration model, TLS handshake state machine, error mapping.
  - Implement: SMTP listener with STARTTLS advertisement and upgrade path.
  - Test: handshake success/failure cases and protocol compliance tests.
  - Docs: listener setup and certificate requirements.
  - Review: security and interoperability review sign-off.

## Context Summary

- Repository started as docs-first plan with no `verzola-proxy` crate.
- Unit U1 targets inbound behavior before outbound relay and policy engine.
- STARTTLS behavior must be explicit and deterministic to avoid downgrade ambiguity.
- Acceptance evidence for this bolt is primarily protocol-level tests.

## File-Level Plan and Outputs

- Created:
  - `verzola-proxy/Cargo.toml`
  - `verzola-proxy/src/lib.rs`
  - `verzola-proxy/src/main.rs`
  - `verzola-proxy/src/inbound/mod.rs`
  - `verzola-proxy/tests/inbound_starttls.rs`
  - `docs/inbound-listener.md`
  - `docs/adr/0001-u1-b1-listener-starttls-state-machine.md`
  - `docs/reviews/u1-b1-security-interoperability.md`

## Acceptance Run

- Command:
  - `cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && cd /d d:\Programming\Repositories\verzola\verzola-proxy && cargo test'`
- Result:
  - passed (`3 passed; 0 failed`).
- Completed:
  - `2026-02-16`

## NFR/Risk Notes

- Security:
  - explicit SMTP status mapping for protocol misuse and temporary TLS failure.
- Reliability:
  - bounded line length and deterministic state transitions.
- Interoperability:
  - RFC-style multiline `EHLO` response formatting and STARTTLS flow enforcement.
