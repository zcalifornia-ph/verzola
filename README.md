<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Apache-2.0][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- ABOUT THE PROJECT -->
##
[![VERZOLA Screen Shot][product-screenshot]](https://github.com/zcalifornia-ph/verzola)
##



<div align="center">
<h3 align="center">VERZOLA</h3>

  <p align="center">
    <strong>VERZOLA is a drop-in SMTP security sidecar for Postfix that prefers hybrid/PQ TLS when possible, falls back safely when not, and makes transport security observable and policy-controlled.</strong>
    <br />
    Version: v0.1.5
    <br />
    Status: pre-alpha (docs/spec complete, implementation in progress).
    <br />
    <a href="https://github.com/zcalifornia-ph/verzola"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/zcalifornia-ph/verzola">View Demo</a>
    &middot;
    <a href="https://github.com/zcalifornia-ph/verzola/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/zcalifornia-ph/verzola/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#what-verzola-is">What VERZOLA Is</a></li>
        <li><a href="#what-verzola-is-not">What VERZOLA Is Not</a></li>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#architecture">Architecture</a>
      <ul>
        <li><a href="#deployment-modes">Deployment Modes</a></li>
        <li><a href="#the-verzola-capable-switch">The VERZOLA-Capable Switch</a></li>
        <li><a href="#data-plane-design">Data Plane Design</a></li>
        <li><a href="#control-plane-design">Control Plane Design</a></li>
        <li><a href="#postfix-integration-plan">Postfix Integration Plan</a></li>
      </ul>
    </li>
    <li>
      <a href="#tls-policy-and-capability-detection">TLS Policy and Capability Detection</a>
    </li>
    <li><a href="#observability-and-evidence">Observability and Evidence</a></li>
    <li><a href="#security-threat-model">Security Threat Model</a></li>
    <li><a href="#repository-plan">Repository Plan</a></li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#quick-start-early-implementation">Quick Start (Early Implementation)</a></li>
      </ul>
    </li>
    <li>
      <a href="#usage">Usage</a>
      <ul>
        <li><a href="#policy-yaml-draft-for-verzolactl">Policy YAML Draft for verzolactl</a></li>
        <li><a href="#postfix-maincfmastercf-snippets-draft">Postfix main.cf/master.cf Snippets (Draft)</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#demo-plan">Demo Plan</a></li>
    <li><a href="#immediate-next-actions">Immediate Next Actions</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## About The Project

VERZOLA is a mail transport sidecar that runs in front of and/or behind Postfix to harden SMTP transport without replacing your existing MTA stack.

Inspired by the life and works of Roberto S. Verzola, widely cited as the "Father of Philippine email" for early email/internet connectivity work supporting NGOs in the Philippines.

### What VERZOLA Is

* A mail transport sidecar for Postfix that terminates and initiates STARTTLS.
* A crypto-agility layer that prefers hybrid/PQ-capable TLS handshakes when peers support them.
* A policy engine surface that supports `opportunistic`, `require-tls`, and `require-pq` (allowlist) behavior.
* An observability layer that exposes metrics and structured logs so transport outcomes are auditable.

### What VERZOLA Is Not

* Not end-to-end email encryption; SMTP remains hop-by-hop.
* Not a replacement for Postfix, Dovecot, or Rspamd.
* Not "PQ-secure email for everyone" today; ecosystem support is still evolving.
* Not a spam filter.

### Built With

* [![Rust][Rust-lang]][Rust-url]
* [![Python][Python]][Python-url]
* [![Postfix][Postfix]][Postfix-url]
* [![Docker][Docker]][Docker-url]
* [![Prometheus][Prometheus]][Prometheus-url]
* [![Grafana][Grafana]][Grafana-url]
* [![OpenSSL][OpenSSL]][OpenSSL-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Architecture

### Deployment Modes

**Mode A - Inbound TLS fronting (receive hardening)**

```text
Internet MTAs -> [VERZOLA :25/:587 (STARTTLS, PQ-prefer)] -> Postfix :2525 (plaintext loopback)
```

**Mode B - Outbound smart relay (send hardening)**

```text
Postfix (relayhost=127.0.0.1:10025) -> [VERZOLA outbound relay] -> Remote MX (STARTTLS, PQ-prefer)
```

**Mode C - Both (recommended)**

* Inbound protection plus outbound crypto agility with unified observability.

### The VERZOLA-Capable Switch

VERZOLA attempts hybrid/PQ TLS negotiation first.

* If remote supports it: hybrid/PQ is negotiated, counted, and logged.
* If not: VERZOLA falls back to classical TLS and still delivers based on policy.

### Data Plane Design

Inbound proxy behavior (fronting Postfix):

* Speaks SMTP to internet clients.
* Advertises STARTTLS and upgrades on request.
* Forwards SMTP commands and DATA to Postfix over localhost.
* Streams message DATA without large buffering.

Outbound relay behavior (Postfix -> VERZOLA -> remote MX):

* Accepts a message from Postfix.
* Attempts immediate remote MX delivery using TLS policy.
* Returns `250` to Postfix only if remote accepts.
* Returns `4xx` on temporary or policy-based delivery failures so Postfix retains queue/retry control.

### Control Plane Design

A separate control component manages:

* Policy config (`YAML`/`TOML`) and schema validation.
* Allowlists (`require-pq`, `require-tls`) and per-domain rules.
* Config generation for Postfix-side integration.
* Dashboards, reports, and audit-friendly operational outputs.

### Postfix Integration Plan

Inbound wiring:

* Postfix listens on `localhost:2525`.
* VERZOLA listens on `:25`/`:587` and forwards locally to Postfix.

Outbound wiring:

* Postfix sets `relayhost = [127.0.0.1]:10025`.
* VERZOLA listens on `10025`, performs remote delivery, and reports status back using `250` or `4xx` semantics.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## TLS Policy and Capability Detection

Policy modes:

1. `opportunistic` (default)
   Prefer hybrid/PQ, then classical TLS, then plaintext only if explicitly allowed by admin policy.
2. `require-tls`
   Require TLS (classical acceptable). If TLS is unavailable, defer or reject per listener policy.
3. `require-pq` (allowlist only)
   For selected partner domains, if negotiation is not hybrid/PQ, return `4xx` (defer) to preserve retries.

Capability detection:

* Primary source: TLS handshake outcome (did a hybrid/PQ group negotiate?).
* Optional source: DNS TXT hint for optimization and policy clarity.

```text
_verzola._tcp.example.com TXT "v=1; mode=hybrid; groups=p256_kyber768"
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Observability and Evidence

Prometheus metrics:

* `verzola_tls_sessions_total{direction=..., result=...}`
* `verzola_pq_negotiated_total{group=...}`
* `verzola_tls_fallback_total{reason=...}`
* `verzola_delivery_attempts_total{status=2xx/4xx/5xx}`
* `verzola_handshake_latency_seconds_bucket`

Structured JSON logs should include:

* Peer domain, MX host, TLS version, cipher, negotiated group, `pq=true/false`, policy decision, and failure reason.

Grafana dashboard focus:

* PQ negotiation rate over time.
* TLS fallback reasons.
* Top partner domains using hybrid/PQ.
* Handshake error spikes.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Security Threat Model

Threats addressed:

* Downgrade attempts via explicit policy controls and allowlists.
* Weak transport security via enforceable TLS requirements.
* Silent delivery/security failures via metrics and logs.
* Configuration drift via generated config plus policy validation.

Threats out of scope:

* End-to-end confidentiality of mail content (requires S/MIME or PGP).
* Remote server compromise.
* User endpoint compromise.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Repository Plan

```text
verzola/
  README.md
  LICENSE
  SECURITY.md
  CONTRIBUTING.md
  CODE_OF_CONDUCT.md

  docs/
    architecture.md
    threat-model.md
    pq-mode.md
    postfix-integration.md
    demo.md
    adr/
    diagrams/
  learn/
    u1-b1-inbound-starttls-study-guide.md
    u1-b2-streaming-forwarder-study-guide.md
    u1-b3-inbound-policy-telemetry-study-guide.md

  deploy/
    compose/
      inbound-only/
      outbound-only/
      full-stack/
    helm/
      verzola/
    ansible/
      roles/

  verzola-proxy/
    Cargo.toml
    src/
      main.rs
      inbound/
      outbound/
      tls/
      smtp/
      metrics/
      config/
    tests/
      integration/

  verzola-control/
    pyproject.toml
    verzola_control/
      cli.py
      policy/
      render/
      validate/
      reports/
    tests/

  dashboards/
    grafana/
      verzola-overview.json

  scripts/
    build-images.sh
    run-demo.sh
    gen-certs.sh
```

Ownership split:

* `verzola-proxy`: SMTP protocol handling, streaming, TLS negotiation, metrics endpoint.
* `verzola-control`: `verzolactl` policy validation, config generation, and reporting.
* `deploy`: one-command demo environments.
* `docs`: architecture and talk-ready technical material.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

Status: pre-alpha (docs/spec complete, implementation in progress).

### Prerequisites

* Postfix (for local integration tests)
* Docker and Docker Compose
* Rust toolchain (`stable`)
* Python 3.11+
* Prometheus and Grafana (for observability stack demos)

### Quick Start (Early Implementation)

1. Clone the repo.
   ```sh
   git clone https://github.com/zcalifornia-ph/verzola.git
   cd verzola
   ```
2. Ensure the Rust toolchain is installed and available on PATH.
   ```sh
   cargo --version
   ```
   On Windows, if you get `'cargo' is not recognized`, add Cargo to the current shell and verify again:
   ```powershell
   $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
   cargo --version
   ```
   If that works, persist it for future shells and then reopen your terminal:
   ```powershell
   $cargoBin = "$env:USERPROFILE\.cargo\bin"
   $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
   if ($userPath -notlike "*$cargoBin*") {
     [Environment]::SetEnvironmentVariable("Path", "$cargoBin;$userPath", "User")
   }
   ```
3. Run inbound STARTTLS tests:
   ```sh
   cd verzola-proxy
   cargo test
   ```
4. Review inbound implementation notes in `docs/inbound-listener.md`, `docs/inbound-postfix-integration.md`, `docs/inbound-policy-telemetry.md`, and ADRs `docs/adr/0001-u1-b1-listener-starttls-state-machine.md` + `docs/adr/0002-u1-b2-streaming-forwarder.md` + `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`.
5. Study the guided walkthroughs in `learn/u1-b1-inbound-starttls-study-guide.md`, `learn/u1-b2-streaming-forwarder-study-guide.md`, and `learn/u1-b3-inbound-policy-telemetry-study-guide.md`.
6. Continue with Unit U2 Bolt U2-B1 (outbound session orchestration).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

### Policy YAML Draft for `verzolactl`

```yaml
version: 1
listeners:
  inbound:
    mode: opportunistic
    allow_plaintext: false
  outbound:
    mode: opportunistic
    allow_plaintext: false

domains:
  partner.example:
    mode: require-pq
    on_mismatch: defer
  legacy.example:
    mode: require-tls

capability_hints:
  dns_txt:
    enabled: true
    label: _verzola._tcp
```

### Postfix `main.cf`/`master.cf` Snippets (Draft)

Inbound (VERZOLA fronts public SMTP, Postfix receives only loopback):

```ini
# main.cf
inet_interfaces = loopback-only
```

```ini
# master.cf
2525      inet  n       -       n       -       -       smtpd
```

Outbound (Postfix queues and retries, VERZOLA performs immediate remote attempts):

```ini
# main.cf
relayhost = [127.0.0.1]:10025
```

Delivery semantics expected from VERZOLA relay:

* Return `250` only when remote MX accepted the message.
* Return `4xx` on temporary failure or policy failure to keep retries in Postfix.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] Phase 0 - Foundation (monorepo scaffold, CI, `verzolactl validate`, demo skeleton)
- [ ] Phase 1 - Inbound proxy (classical TLS, STARTTLS termination, streaming DATA, metrics/logs)
- [ ] Phase 2 - Outbound relay (remote MX delivery, correct `250/4xx` behavior, metrics/logs)
- [ ] Phase 3 - Policy-as-code and guardrails (domain rules, optional DNS hint, docs/diagrams)
- [ ] Phase 4 - PQ lab mode (hybrid/PQ preference with experimental TLS stack)
- [ ] Phase 5 - Hardening and release polish (least privilege, security docs, reproducible demo, tagged release)

Progress note: Unit U1 Bolts U1-B1, U1-B2, and U1-B3 are complete in `REQUIREMENTS.md`, with listener + streaming relay + inbound policy/telemetry implementation, integration tests, and docs landed (`verzola-proxy/src/inbound/*`, `verzola-proxy/tests/inbound_starttls.rs`, `verzola-proxy/tests/inbound_forwarder.rs`, `verzola-proxy/tests/inbound_policy_telemetry.rs`, `docs/*`).

Learning note: step-by-step learning assets for Unit U1 are available at `learn/u1-b1-inbound-starttls-study-guide.md`, `learn/u1-b2-streaming-forwarder-study-guide.md`, and `learn/u1-b3-inbound-policy-telemetry-study-guide.md`.

See the [open issues](https://github.com/zcalifornia-ph/verzola/issues) for proposed features and known gaps.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Demo Plan

Two docker-compose stacks:

1. Normal mail server stack (classical TLS only).
2. VERZOLA-capable stack (PQ lab mode enabled).

Demo flow:

* A -> B delivery negotiates hybrid/PQ (verified via metrics/logs).
* Delivery to non-capable server safely falls back to classical TLS.
* Strict allowlist policy blocks downgrade via defer/retry behavior.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



## Immediate Next Actions

1. Implement Unit U2 Bolt U2-B1 outbound session orchestration.
2. Add production TLS adapter wiring (`TlsUpgrader`) with certificate loading, secure defaults, and clear failure mapping.
3. Add CI checks to run proxy lint/test gates on every pull request.
4. Add inbound interoperability checks using a real SMTP client matrix (for example Postfix and swaks).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are welcome, especially around SMTP interoperability tests, policy validation tooling, and observability improvements.
See `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md` for process, behavior, and vulnerability reporting.

1. Fork the project.
2. Create your feature branch (`git checkout -b feature/your-feature`).
3. Commit your changes (`git commit -m 'Add some feature'`).
4. Push to your branch (`git push origin feature/your-feature`).
5. Open a pull request.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/zcalifornia-ph/verzola/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=zcalifornia-ph/verzola" alt="contrib.rocks image" />
</a>



<!-- LICENSE -->
## License

Distributed under the Apache-2.0 license. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Maintainer - [@zcalifornia_](https://twitter.com/zcalifornia_) - zecalifornia@up.edu.ph

Project Link: [https://github.com/zcalifornia-ph/verzola](https://github.com/zcalifornia-ph/verzola)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* Roberto S. Verzola, for pioneering early Philippine NGO email/internet connectivity.
* Postfix maintainers and operators whose reliability model makes sidecar integration practical.
* Open-source observability ecosystem (Prometheus + Grafana) for operational transparency.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/zcalifornia-ph/verzola.svg?style=for-the-badge
[contributors-url]: https://github.com/zcalifornia-ph/verzola/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/zcalifornia-ph/verzola.svg?style=for-the-badge
[forks-url]: https://github.com/zcalifornia-ph/verzola/network/members
[stars-shield]: https://img.shields.io/github/stars/zcalifornia-ph/verzola.svg?style=for-the-badge
[stars-url]: https://github.com/zcalifornia-ph/verzola/stargazers
[issues-shield]: https://img.shields.io/github/issues/zcalifornia-ph/verzola.svg?style=for-the-badge
[issues-url]: https://github.com/zcalifornia-ph/verzola/issues
[license-shield]: https://img.shields.io/github/license/zcalifornia-ph/verzola.svg?style=for-the-badge
[license-url]: https://github.com/zcalifornia-ph/verzola/blob/main/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/zcalifornia
[product-screenshot]: repo/images/verzola-screen.png
[Rust-lang]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Python]: https://img.shields.io/badge/Python-3776AB?style=for-the-badge&logo=python&logoColor=white
[Python-url]: https://www.python.org/
[Postfix]: https://img.shields.io/badge/Postfix-MTA-00599C?style=for-the-badge
[Postfix-url]: http://www.postfix.org/
[Docker]: https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white
[Docker-url]: https://www.docker.com/
[Prometheus]: https://img.shields.io/badge/Prometheus-E6522C?style=for-the-badge&logo=prometheus&logoColor=white
[Prometheus-url]: https://prometheus.io/
[Grafana]: https://img.shields.io/badge/Grafana-F46800?style=for-the-badge&logo=grafana&logoColor=white
[Grafana-url]: https://grafana.com/
[OpenSSL]: https://img.shields.io/badge/OpenSSL-721412?style=for-the-badge&logo=openssl&logoColor=white
[OpenSSL-url]: https://www.openssl.org/
