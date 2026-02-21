# Contributing

Status: pre-alpha (docs/spec complete, implementation in progress).

Thanks for helping improve VERZOLA.

## Before You Start

- Check existing issues and open pull requests before starting work.
- For significant changes, open an issue first to align on scope.
- Keep pull requests focused and reviewable.
- Confirm the active milestone scope in `REQUIREMENTS.md` before implementing.

## Development Workflow

1. Fork the repository and create a branch.
2. Make your changes with tests and docs updates when applicable.
3. Run validation locally from `verzola-proxy`:
   - `cd verzola-proxy`
   - `cargo test`
   - optional targeted suites:
     - `cargo test --test inbound_starttls`
     - `cargo test --test inbound_forwarder`
     - `cargo test --test inbound_policy_telemetry`
     - `cargo test --test outbound_orchestration`
     - `cargo test --test outbound_status_contract`
     - `cargo test --test outbound_tls_policy`
4. Run control-plane validation from `verzola-control`:
   - `cd ../verzola-control`
   - `python -B -m unittest discover -s tests -v`
   - optional CLI check:
     - `python -m verzola_control validate <policy-file.yaml>`
     - `python -m verzola_control render <policy-file.yaml> --environment dev --output -`
5. Open a pull request with:
   - problem statement
   - approach summary
   - validation evidence (test output, screenshots, logs)

## Documentation Sync Requirements

- If you change milestone status, update `REQUIREMENTS.md`, `README.md`, and `CHANGELOG.md` in the same PR.
- If you add or rename technical docs, update `README.md` references and related `docs/`/`learn/` links.
- Do not delete generated artifacts in docs-task automation; list manual cleanup targets under `CHANGELOG.md` `For Deletion`.

## Commit Messages

Use Conventional Commits (for example `docs:`, `feat:`, `fix:`, `chore:`).

## Code of Conduct

Participation in this project means following `CODE_OF_CONDUCT.md`.

## Security

For vulnerability reports, follow `SECURITY.md`.
