# Security Policy

Status: pre-alpha (docs/spec complete, implementation in progress).

## Supported Versions

VERZOLA is currently in pre-alpha (`0.x`).
There are no long-term support branches yet.
Security fixes are applied on `main` and documented in `CHANGELOG.md`.

## Current Security Scope and Limitations

- Implemented and test-covered scope includes:
  - inbound proxy slice in `verzola-proxy` (Unit U1 Bolts U1-B1/U1-B2/U1-B3),
  - outbound session orchestration slice (Unit U2 Bolt U2-B1).
- A production TLS adapter is not yet implemented; current code uses a `TlsUpgrader` interface with `NoopTlsUpgrader` for scaffolding and tests.
- Outbound delivery status contract/policy enforcement, control-plane policy tooling, and full observability pipeline remain planned (`REQUIREMENTS.md` Units U2-B2 and beyond).
- Treat current builds as pre-production and validate controls in an isolated environment before any internet-facing deployment.

## Reporting a Vulnerability

Use coordinated disclosure. Do not open a public issue for a suspected vulnerability.

Send a report to `zecalifornia@up.edu.ph` with:
- affected component or file path
- reproduction steps or proof of concept
- expected impact and severity
- suggested remediation (optional)

## Response Targets

- acknowledgment within 72 hours
- initial triage within 7 days
- best-effort fix or mitigation plan within 30 days

## Disclosure and Credit

After a fix is available (or risk is accepted), disclosure timing will be coordinated with the reporter.
Reporter credit will be provided unless anonymity is requested.
