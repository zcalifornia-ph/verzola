# Version v0.1.14 Documentation

## Title
Documentation Hygiene Release: Changelog Path Sanitization and Version Sync

## Quick Diagnostic Read

This version is a documentation-only sync release.

Primary outcomes:

- historical `CHANGELOG.md` cleanup removes a hidden local temp-path reference from prior `For Deletion` notes,
- `README.md` version metadata advances to `v0.1.14`,
- a new version detail note captures traceability for this docs hygiene change.

No code, tests, or milestone completion status changed in this version.

## One-Sentence Objective

Preserve documentation hygiene and release traceability by removing hidden local-path references while keeping root version metadata in sync.

## Scope of This Version

This version includes:

- historical changelog cleanup for `v0.1.11` through `v0.1.13`,
- root README version/snapshot synchronization,
- a new version documentation artifact,
- refreshed manual cleanup notes for currently present workspace build/probe artifacts.

## Detailed Changes

## 1) Historical Changelog Cleanup

Updated:

- `CHANGELOG.md`

What changed:

- Replaced a hidden local temp-path reference in `v0.1.11`, `v0.1.12`, and `v0.1.13` `### For Deletion` sections with generic wording.
- Preserved the original intent (manual cleanup guidance) without exposing hidden/local workflow details.

## 2) Release Metadata Sync (`v0.1.14`)

Updated:

- `README.md`
- `CHANGELOG.md`

Sync points:

- `README.md` now shows `v0.1.14` in the main version marker.
- `README.md` repository snapshot now includes `docs/version-v0.1.14-docs.md`.
- `CHANGELOG.md` now includes a `v0.1.14` entry describing this documentation-only release.

## 3) Manual Cleanup Visibility (Non-Destructive)

Updated:

- `CHANGELOG.md`

What was documented:

- current workspace cargo target directories and probe artifacts under `repo/` and `verzola-proxy/`,
- listed under `### For Deletion` for manual cleanup only.

Important note:

- No files were deleted as part of this task.

## 4) Version Documentation Artifact

Added:

- `docs/version-v0.1.14-docs.md`

Purpose:

- record the rationale, scope, and validation notes for this docs hygiene release beyond the summary bullets in `CHANGELOG.md`.

## Traceability Links

- Root documentation baseline:
  - `README.md`
  - `CHANGELOG.md`
  - `REQUIREMENTS.md`
- Prior version detail note (for continuity):
  - `docs/version-v0.1.13-docs.md`

## Validation Notes

Validation checks:

- root-doc scan for hidden workflow and local temp-path references (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`)
- diff review for doc-only scope (`README.md`, `CHANGELOG.md`, `docs/version-v0.1.14-docs.md`)

Observed result:

- no hidden workflow or local temp-path references remain in the target root docs,
- changes are limited to documentation files and release metadata sync.

Runtime tests:

- not run (documentation-only change)

Validation date:

- `2026-02-23`
