# Version v0.1.6 Documentation

## Title
Repository Reality Sync and Governance Documentation Alignment

## Quick Diagnostic Read

This version is documentation-focused and resolves drift between planned architecture text and the repository's current implementation state.

Primary outcomes:

- root documentation now distinguishes implemented scope (Unit U1) from planned scope (Units U2-U6),
- governance docs now describe current security and documentation-update expectations more explicitly,
- cleanup candidates for generated artifacts are recorded without deleting files.

## One-Sentence Objective

Align root markdown documentation with the actual workspace state so contributors can trust version status, repository structure, and current security/operational boundaries.

## Scope of This Version

This version updates documentation only:

- `README.md` (version marker, repository snapshot, test validation context),
- `CHANGELOG.md` (new version entry and manual deletion list),
- `CONTRIBUTING.md` (documentation-sync workflow and validation command coverage),
- `SECURITY.md` (current pre-alpha scope and limitations),
- this detailed version note in `docs/version-v0.1.6-docs.md`.

No application logic or test code behavior was intentionally changed in this version.

## Detailed Changes

## 1) README Reality Sync

Updated:

- version marker to `v0.1.6`,
- section title from `Repository Plan` to `Repository Snapshot`,
- root tree content to reflect currently present files/directories,
- explicit note separating planned expansion from implemented scope,
- quick-start validation with both full suite and targeted inbound test commands,
- roadmap validation note with dated inbound suite pass summary.

Reason:

- prevent contributor confusion caused by documentation that implied non-existent directories/modules were already present.

## 2) CONTRIBUTING and SECURITY Clarifications

Updated `CONTRIBUTING.md`:

- added requirement to verify active milestone scope in `REQUIREMENTS.md`,
- expanded local validation commands for inbound integration suites,
- added explicit documentation-sync requirements across `REQUIREMENTS.md`, `README.md`, and `CHANGELOG.md`.

Updated `SECURITY.md`:

- documented implemented/test-covered scope as Unit U1 inbound proxy slices,
- clarified that production TLS adapter wiring is still pending,
- clarified that U2-U6 security surface remains planned and pre-production caution applies.

Reason:

- keep contributor and security expectations aligned with the current maturity level of the project.

## 3) Changelog and Cleanup Traceability

Updated `CHANGELOG.md`:

- added `v0.1.6` release notes covering all documentation realignment work,
- kept `### For Deletion` focused on existing generated artifact directories that require manual cleanup.

Reason:

- preserve release traceability while respecting task rules that prohibit automated deletion.

## 4) Version Documentation Artifact

Added:

- `docs/version-v0.1.6-docs.md` (this file)

Reason:

- maintain a detailed per-version narrative beyond concise changelog bullets.

## Traceability Links

- Requirements baseline:
  - `REQUIREMENTS.md`
- Root docs updated in this version:
  - `README.md`
  - `CHANGELOG.md`
  - `CONTRIBUTING.md`
  - `SECURITY.md`
- Prior version narrative:
  - `docs/version-v0.1.5-docs.md`

## Validation Notes

Consistency and evidence checks performed:

- confirmed deletion-list paths in `CHANGELOG.md` exist in workspace at update time,
- verified `README.md` table-of-contents anchor changed to `#repository-snapshot`,
- validated inbound test evidence via:
  - `cargo test` in `verzola-proxy`,
  - passing integration suites: `inbound_forwarder` (2), `inbound_policy_telemetry` (4), `inbound_starttls` (3),
  - validation run date: `2026-02-17`.

## Practical Next Use

Recommended contributor sequence after this version:

1. Review `REQUIREMENTS.md` to confirm current milestone boundary.
2. Use `README.md` quick-start commands to run inbound validation.
3. Follow `CONTRIBUTING.md` documentation-sync rules when editing roadmap/status docs.
4. Proceed to Unit U2 implementation tasks using clarified scope boundaries.

