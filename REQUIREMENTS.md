# REQUIREMENTS

## Task Inputs
- Input 1 (output location): repository root (`d:\Programming\Repositories\verzola`)
- Input 2 (task to decompose): `README.md` (VERZOLA sidecar scope and roadmap)

## Intent (Level 0)
Deliver VERZOLA as a production-credible Postfix sidecar that applies policy-controlled SMTP transport hardening with hybrid/PQ TLS preference, safe fallback behavior, and auditable operational evidence.

## Assumptions + Questions

### Assumptions
1. The first release target is Linux-based deployments where Postfix is already in use.
2. Hybrid/PQ TLS support is treated as experimental/lab-mode in early releases, with classical TLS as the default interoperability baseline.
3. Postfix retains queue ownership and retry behavior; VERZOLA must not become a durable queue.
4. `verzola-proxy` is implemented in Rust and `verzola-control` in Python as described in `README.md`.
5. Initial deployment focus is Docker Compose demo stacks, then Kubernetes/Helm readiness.
6. Operational teams require Prometheus-compatible metrics and JSON logs from day one.
7. Delivery policy for `require-pq` mismatch is defer (`4xx`) by default, not permanent fail (`5xx`), unless policy explicitly states otherwise.

### Clarifying Questions
1. Which minimum Postfix version is the support baseline for integration and test matrices?
2. Which TLS library/stack is approved for hybrid/PQ experimentation in Phase 4?
3. Should DNS TXT capability hints be advisory-only, or allowed to enforce stricter behavior when present?
4. Which compliance profile is required for first production deployment (for example, ISO 27001 controls, SOC 2 evidence)?
5. What SLO target should be committed for message delivery latency impact vs direct Postfix relay?

Proceeding with the assumptions above until clarifications are provided.

## Inception

### User Stories With Acceptance Criteria

#### US-01 Inbound STARTTLS Fronting
As a mail operator, I want VERZOLA to front inbound SMTP and forward to loopback Postfix so public ingress is transport-hardened without replacing Postfix.

Acceptance criteria:
- [ ] VERZOLA accepts SMTP sessions on configured listener ports (`25`, optional `587`).
- [ ] STARTTLS is advertised and TLS upgrade succeeds for standards-compliant clients.
- [ ] Mail accepted by VERZOLA is relayed to Postfix on loopback (`localhost:2525`) without full-message buffering.
- [ ] For non-upgrade sessions, behavior matches configured policy (`opportunistic`, `require-tls`, `require-pq` for allowlisted domains).
- [ ] Session outcomes are emitted as metrics and structured logs.

#### US-02 Outbound Smart Relay Semantics
As a mail operator, I want Postfix to relay outbound mail through VERZOLA so transport policy is applied while Postfix keeps queue/retry control.

Acceptance criteria:
- [ ] Postfix can route via `relayhost = [127.0.0.1]:10025`.
- [ ] VERZOLA attempts immediate remote MX delivery for each message received from Postfix.
- [ ] VERZOLA returns `250` only after remote MX acceptance.
- [ ] VERZOLA returns `4xx` for temporary or policy-based deferrals, preserving Postfix retries.
- [ ] Outbound TLS negotiation result and policy decision are logged and metered.

#### US-03 Policy-As-Code Management
As a platform engineer, I want declarative policy configuration so transport requirements are reviewable, validated, and reproducible.

Acceptance criteria:
- [ ] `verzolactl validate` rejects malformed config and unknown policy values.
- [ ] Domain policy overrides support `opportunistic`, `require-tls`, and `require-pq`.
- [ ] Policy renderer generates deterministic effective config artifacts.
- [ ] Policy changes are traceable via versioned files and validation output.

#### US-04 Crypto Agility With Safe Fallback
As a security owner, I want hybrid/PQ preference when peers support it and safe classical fallback when they do not.

Acceptance criteria:
- [ ] TLS negotiation attempts hybrid/PQ-capable groups first when enabled.
- [ ] If hybrid/PQ is unavailable, classical TLS is attempted unless policy forbids.
- [ ] If policy requires TLS and no TLS is possible, message handling follows configured defer/reject behavior.
- [ ] Negotiated group and fallback reason are observable per session.

#### US-05 Operational Evidence and Dashboards
As an operations team, I want immediate visibility into transport security posture to detect downgrades and handshake failures.

Acceptance criteria:
- [ ] Prometheus endpoint exposes required counters and latency histograms.
- [ ] Logs are JSON-formatted and include domain, MX host, TLS version/cipher/group, policy decision, and failure reason.
- [ ] Grafana dashboard includes PQ negotiation rate, fallback reasons, handshake errors, and domain-level trends.
- [ ] Demo environment shows evidence for both PQ-capable and non-capable peers.

#### US-06 Security and Release Readiness
As a maintainer, I want hardened defaults and release artifacts so deployments are safe and operable.

Acceptance criteria:
- [ ] Listener and control-plane defaults follow least-privilege and secure-by-default posture.
- [ ] Threat model, security policy, and operational runbooks exist and are versioned.
- [ ] CI validates core lint/test/security checks before release tagging.
- [ ] Release notes include known limitations and supported deployment modes.

### Non-Functional Requirements (NFRs)

#### Security
- NFR-SEC-01: All public listeners must support TLS 1.2+; TLS 1.0/1.1 are disabled.
- NFR-SEC-02: `require-tls` and `require-pq` policy violations must produce deterministic `4xx`/`5xx` outcomes per configuration.
- NFR-SEC-03: Control-plane actions (validate/render/report) must produce auditable logs with actor and timestamp.

#### Privacy
- NFR-PRV-01: Logs must not include full message bodies or credentials.
- NFR-PRV-02: Sensitive fields in logs/config dumps must be redacted (for example, auth secrets, private keys).
- NFR-PRV-03: Data retention defaults for logs/metrics must be documented with configurable limits.

#### Performance
- NFR-PERF-01: Inbound proxy P95 handshake latency overhead <= 150 ms compared to direct Postfix in local benchmark conditions.
- NFR-PERF-02: Outbound relay processing overhead per message (excluding remote MX latency) <= 100 ms P95 in controlled test.
- NFR-PERF-03: Memory footprint per active SMTP session stays within documented limits under stress test.

#### Reliability
- NFR-REL-01: Service availability target >= 99.9% monthly for single-instance demo baseline with restart policy.
- NFR-REL-02: On transient remote errors, VERZOLA must return `4xx` and never lose accepted Postfix queue ownership semantics.
- NFR-REL-03: Configuration reloads must be atomic and must not drop active sessions unexpectedly.

#### Cost
- NFR-COST-01: Default deployment must run on a small VM profile (documented CPU/RAM baseline).
- NFR-COST-02: Optional PQ mode must be feature-gated to avoid unnecessary compute spend on unsupported peers.

#### UX (Operator Experience)
- NFR-UX-01: Validation errors must be actionable, with file path, field path, and suggested correction.
- NFR-UX-02: Default dashboards must provide at-a-glance status without custom query authoring.
- NFR-UX-03: Quick-start setup to first successful inbound and outbound test should be <= 30 minutes with prerequisites met.

#### Compliance
- NFR-CMP-01: Security-relevant changes must be traceable via version control and changelog entries.
- NFR-CMP-02: Threat model and security policy documentation must be present before 1.0 GA (during the 0.x pre-release cycle).
- NFR-CMP-03: Release artifacts must include checksums and provenance notes for reproducibility.

### Risk Register

| Risk | Impact | Mitigation | Owner |
|---|---|---|---|
| Hybrid/PQ interop instability across MTAs/TLS stacks | Delivery failures or frequent fallback | Keep PQ mode feature-gated; maintain compatibility test matrix; default to classical TLS fallback unless strict policy | Security Engineering |
| Incorrect `250/4xx` relay semantics | Queue inconsistency or message loss risk | Contract tests for Postfix relay semantics; canary rollout with message trace audits | Proxy Engineering |
| Policy misconfiguration by operators | Unintended plaintext delivery or false deferrals | Strong schema validation, safe defaults, lint rules, and preflight checks | Control Plane Engineering |
| High-cardinality metrics/log labels | Monitoring cost and degraded observability stack performance | Label budget policy, sampling, bounded dimensions, and dashboard review gates | SRE/Observability |
| Incomplete threat model coverage | Security gaps entering production | Security review checkpoints per phase and mandatory threat-model updates before release | Security Lead |
| Deployment drift between docs and manifests | Failed installs and operational errors | Single-source generated configs, versioned examples, and CI validation of deploy assets | Release Engineering |

### Human Validation Required (After Inception Outputs)
- [ ] Human validation required: approve Intent, User Stories, NFRs, and Risk Register before unit execution.

## Units (Main Work Packages)

### [x] Unit U1: Inbound SMTP Proxy
Scope:
- Implement inbound listener, STARTTLS handling, SMTP forwarding, and policy enforcement for inbound traffic.

Interfaces/Dependencies:
- Postfix loopback listener (`localhost:2525`)
- TLS stack configuration and certificate material
- Metrics/logging sink

Acceptance criteria:
- [x] SMTP ingress works on configured ports with STARTTLS upgrade.
- [x] Streaming relay to Postfix works without large-buffer memory spikes.
- [x] Policy outcomes are deterministic and observable.

Deliverables:
- [x] `verzola-proxy/src/inbound/*`
- [x] Integration tests for inbound flows
- [x] Operator docs for inbound mode

#### [x] Bolt U1-B1: Listener and STARTTLS Negotiation
Subtasks:
- [x] Design: listener configuration model, TLS handshake state machine, error mapping.
- [x] Implement: SMTP listener with STARTTLS advertisement and upgrade path.
- [x] Test: handshake success/failure cases and protocol compliance tests.
- [x] Docs: listener setup and certificate requirements.
- [x] Review: security and interoperability review sign-off.
- Completed: 2026-02-16
- Evidence: `cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && cd /d d:\Programming\Repositories\verzola\verzola-proxy && cargo test'` -> `3 passed; 0 failed`.

#### [x] Bolt U1-B2: Streaming Forwarder to Postfix
Subtasks:
- [x] Design: streaming pipeline and backpressure behavior.
- [x] Implement: command/data relay to loopback Postfix endpoint.
- [x] Test: large message and concurrent session tests.
- [x] Docs: integration notes for `main.cf` and `master.cf`.
- [x] Review: performance review against memory/latency targets.
- Completed: 2026-02-16
- Evidence: `powershell -Command "$env:PATH=\"$env:USERPROFILE\\.cargo\\bin;$env:PATH\"; cargo test"` -> `5 passed; 0 failed`.

#### [x] Bolt U1-B3: Inbound Policy Enforcement and Telemetry
Subtasks:
- [x] Design: policy decision points and telemetry schema.
- [x] Implement: `opportunistic` and `require-tls` handling for inbound sessions.
- [x] Test: policy matrix tests and telemetry assertion tests.
- [x] Docs: policy behavior reference for inbound paths.
- [x] Review: operational readiness review with SRE.
- Completed: 2026-02-17
- Evidence: `cargo test` -> `inbound_policy_telemetry (4 passed)`, `inbound_starttls (3 passed)`, `inbound_forwarder (2 passed)`.

- [x] Human validation required: approve Unit U1 plan and acceptance criteria.

### [x] Unit U2: Outbound Smart Relay
Scope:
- Implement outbound relay path from Postfix to remote MX with strict `250/4xx` semantics.

Interfaces/Dependencies:
- Postfix relayhost interface (`127.0.0.1:10025`)
- DNS MX resolution and remote SMTP peers
- Policy engine outputs

Acceptance criteria:
- [x] `250` is returned only after confirmed remote acceptance.
- [x] `4xx` is returned on temporary/policy failures requiring retry.
- [x] Outbound mode is configurable independently from inbound mode.

Deliverables:
- [x] `verzola-proxy/src/outbound/*`
- [x] Contract tests for Postfix retry semantics
- [x] Outbound operations guide

#### [x] Bolt U2-B1: Outbound Session Orchestration
Subtasks:
- [x] Design: outbound transaction lifecycle and MX selection strategy.
- [x] Implement: receive from Postfix and establish remote SMTP sessions.
- [x] Test: success path and transient failure handling.
- [x] Docs: outbound mode configuration examples.
- [x] Review: protocol behavior review with mail ops stakeholders.
- Completed: 2026-02-18
- Evidence: `cargo test` -> `11 integration tests passed (including outbound_orchestration: 2 passed; 0 failed)`.

#### [x] Bolt U2-B2: Delivery Status Contract (`250/4xx`)
Subtasks:
- [x] Design: mapping remote outcomes to Postfix-facing statuses.
- [x] Implement: deterministic status mapping and failure classification.
- [x] Test: contract tests for retry-safe semantics.
- [x] Docs: operator expectations and troubleshooting matrix.
- [x] Review: regression review against message safety requirements.
- Completed: 2026-02-19
- Evidence: `cargo test` -> `13 integration tests passed (including outbound_orchestration: 2 passed; outbound_status_contract: 2 passed; 0 failed)`.

#### [x] Bolt U2-B3: Outbound TLS Policy Application
Subtasks:
- [x] Design: outbound policy evaluation order and fallback logic.
- [x] Implement: `opportunistic`, `require-tls`, and per-domain rules.
- [x] Test: policy coverage tests for supported modes.
- [x] Docs: outbound policy examples including defer behavior.
- [x] Review: security review for downgrade resistance.
- Completed: 2026-02-20
- Evidence: `cargo test` -> all suites passed, including `outbound_tls_policy` (6 passed), `outbound_orchestration` (2 passed), and `outbound_status_contract` (2 passed).

- [x] Human validation required: approve Unit U2 plan and acceptance criteria.

### [ ] Unit U3: Policy and Control Plane (`verzolactl`)
Scope:
- Build config schema validation, effective config rendering, and policy reporting workflows.

Interfaces/Dependencies:
- Policy files (`YAML`/`TOML`)
- Proxy config ingestion format
- CI pipeline for validation gates

Acceptance criteria:
- [ ] Invalid policies fail validation with actionable diagnostics.
- [ ] Rendered config is deterministic and environment-aware.
- [ ] Reports summarize domain policy posture and detected gaps.

Deliverables:
- [ ] `verzola-control/verzola_control/policy/*`
- [ ] `verzola-control/verzola_control/validate/*`
- [ ] CLI docs and examples

#### [x] Bolt U3-B1: Schema and Validation Engine
Subtasks:
- [x] Design: schema model and error taxonomy.
- [x] Implement: parser + validator with strict mode.
- [x] Test: malformed/edge-case fixture suite.
- [x] Docs: policy schema reference.
- [x] Review: maintainability review for future policy expansion.
- Completed: 2026-02-21
- Evidence: `python -B -m unittest discover -s tests -v` (run in `verzola-control`) -> `9 passed; 0 failed`.

#### [ ] Bolt U3-B2: Config Renderer
Subtasks:
- [ ] Design: intermediate representation and rendering templates.
- [ ] Implement: deterministic config generation.
- [ ] Test: snapshot tests for renderer outputs.
- [ ] Docs: generated artifact semantics.
- [ ] Review: compatibility review with proxy ingestion.

#### [ ] Bolt U3-B3: Policy Reports and CLI UX
Subtasks:
- [ ] Design: report sections and severity levels.
- [ ] Implement: CLI commands for validate/render/report.
- [ ] Test: CLI integration tests with sample repos.
- [ ] Docs: operator workflow from policy edit to deploy.
- [ ] Review: usability review for actionable errors.

- [ ] Human validation required: approve Unit U3 plan and acceptance criteria.

### [ ] Unit U4: TLS Capability Detection and PQ Mode
Scope:
- Add capability detection mechanisms and optional hybrid/PQ preference controls.

Interfaces/Dependencies:
- TLS negotiation metadata from proxy sessions
- Optional DNS TXT hint lookup
- Policy configuration for strict/allowlist behavior

Acceptance criteria:
- [ ] Negotiated group and PQ/classical result are captured per TLS session.
- [ ] DNS hint support is optional and cannot silently weaken strict policy.
- [ ] `require-pq` allowlist behavior yields documented defer/reject outcomes.

Deliverables:
- [ ] `verzola-proxy/src/tls/*`
- [ ] Capability detection tests
- [ ] PQ mode documentation and limitations

#### [ ] Bolt U4-B1: Negotiation Result Classification
Subtasks:
- [ ] Design: result classification model (`pq`, `classical`, `none`).
- [ ] Implement: parser for negotiated group metadata.
- [ ] Test: classification tests across handshake fixtures.
- [ ] Docs: classification mapping reference.
- [ ] Review: crypto review for correctness.

#### [ ] Bolt U4-B2: DNS TXT Hint Integration (Optional)
Subtasks:
- [ ] Design: hint lookup lifecycle and caching rules.
- [ ] Implement: DNS TXT parser for `_verzola._tcp` label.
- [ ] Test: positive/negative DNS hint scenarios.
- [ ] Docs: advisory semantics and security caveats.
- [ ] Review: threat-model update for DNS spoofing considerations.

#### [ ] Bolt U4-B3: Strict PQ Policy Handling
Subtasks:
- [ ] Design: allowlist evaluation and mismatch actions.
- [ ] Implement: `require-pq` decision flow for partner domains.
- [ ] Test: mismatch behavior and retry-safe outcome validation.
- [ ] Docs: partner policy onboarding guide.
- [ ] Review: partner-facing policy sign-off workflow.

- [ ] Human validation required: approve Unit U4 plan and acceptance criteria.

### [ ] Unit U5: Observability and Operational Evidence
Scope:
- Provide metrics, structured logs, and dashboard assets supporting transport security audits.

Interfaces/Dependencies:
- Prometheus scraping
- Grafana dashboard provisioning
- Log sink (stdout/file/aggregator)

Acceptance criteria:
- [ ] Required metrics are exported with stable names and bounded label sets.
- [ ] Logs include required transport context without sensitive payload leakage.
- [ ] Dashboard panels visualize PQ rate, fallback reasons, and error spikes.

Deliverables:
- [ ] `verzola-proxy/src/metrics/*`
- [ ] `dashboards/grafana/verzola-overview.json`
- [ ] Observability runbook

#### [ ] Bolt U5-B1: Metrics Endpoint and Counters
Subtasks:
- [ ] Design: metric names, labels, and cardinality budget.
- [ ] Implement: counters/histograms per README metric set.
- [ ] Test: scrape tests and metric contract tests.
- [ ] Docs: metric dictionary and alert suggestions.
- [ ] Review: observability review for operational signal quality.

#### [ ] Bolt U5-B2: Structured Logging
Subtasks:
- [ ] Design: JSON schema and field redaction rules.
- [ ] Implement: session-scoped structured logging.
- [ ] Test: schema and redaction tests.
- [ ] Docs: log field reference and example queries.
- [ ] Review: security/privacy review for data exposure risk.

#### [ ] Bolt U5-B3: Grafana Dashboard Package
Subtasks:
- [ ] Design: panel layout for security and reliability outcomes.
- [ ] Implement: dashboard JSON with importable defaults.
- [ ] Test: dashboard smoke test against demo metrics.
- [ ] Docs: dashboard interpretation and response playbooks.
- [ ] Review: ops review for on-call usability.

- [ ] Human validation required: approve Unit U5 plan and acceptance criteria.

### [ ] Unit U6: Deployment, CI, and Release Hardening
Scope:
- Deliver reproducible deployment assets, test automation, and release governance artifacts.

Interfaces/Dependencies:
- Docker/Compose assets
- CI runners and test matrix
- Docs and security governance files

Acceptance criteria:
- [ ] Compose stacks run inbound-only, outbound-only, and full-stack demos.
- [ ] CI gates cover lint, unit/integration tests, and security checks.
- [ ] Release process emits versioned artifacts and notes with known limitations.

Deliverables:
- [ ] `deploy/compose/*`
- [ ] CI pipeline config
- [ ] `SECURITY.md`, runbooks, release checklist

#### [ ] Bolt U6-B1: Demo Deployment Assets
Subtasks:
- [ ] Design: compose topology and service boundaries.
- [ ] Implement: inbound-only, outbound-only, and full-stack manifests.
- [ ] Test: deterministic bring-up/tear-down and health checks.
- [ ] Docs: quick-start demo walkthrough.
- [ ] Review: release engineering review for reproducibility.

#### [ ] Bolt U6-B2: CI/CD Quality Gates
Subtasks:
- [ ] Design: mandatory checks and branch protection requirements.
- [ ] Implement: lint/test/security workflows.
- [ ] Test: failure injection for gate validation.
- [ ] Docs: contributor CI expectations.
- [ ] Review: maintainer sign-off on gate strictness.

#### [ ] Bolt U6-B3: Security and Release Documentation
Subtasks:
- [ ] Design: required security and release artifacts.
- [ ] Implement: threat model, runbooks, and release checklist updates.
- [ ] Test: documentation completeness audit against template.
- [ ] Docs: publish operator handoff package.
- [ ] Review: final readiness review before tagged release.

- [ ] Human validation required: approve Unit U6 plan and acceptance criteria.

## Construction + Operations Coverage

### Design/Architecture (Domain Model + Logical Design Decisions)
- [ ] Define domain entities: `Session`, `Policy`, `DomainRule`, `DeliveryAttempt`, `TlsOutcome`, `EvidenceRecord`.
- [ ] Define bounded contexts: Inbound Proxy, Outbound Relay, Policy Control, Observability.
- [ ] Record ADRs for TLS library choice, fallback semantics, DNS hint usage, and deployment model.
- [ ] Enforce explicit interfaces between contexts (config contracts, telemetry schema, status mapping).

### Testing/Validation (Functional, Security, Performance)
- [ ] Functional: SMTP protocol conformance tests for inbound and outbound paths.
- [ ] Functional: end-to-end integration with Postfix loopback and relayhost modes.
- [ ] Security: downgrade attempt scenarios, strict policy behavior, log redaction checks.
- [ ] Performance: handshake latency, throughput, and memory/concurrency stress tests.
- [ ] Validation: policy schema and renderer snapshot tests in CI.

### Deployment/Operations (Deployment Unit, Observability, Runbooks, Rollback)
- [ ] Deployment unit: containerized proxy + control tooling with explicit config volume boundaries.
- [ ] Observability: default scrape config, dashboard import path, and alert starter set.
- [ ] Runbooks: startup validation, incident triage, and common failure troubleshooting steps.
- [ ] Rollback: version pinning strategy and configuration rollback procedure with validation gate.
- [ ] Change management: staged rollout plan (dev -> staging -> production) with canary checks.
