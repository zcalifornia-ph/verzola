# Version v0.1.5 Documentation

## Title
Learning Asset Expansion for Unit U1 Bolt U1-B3

## Quick Diagnostic Read

This version is documentation-focused and centered on skill transfer after completing U1-B3.

Primary outcome:

- a new study guide now teaches the inbound policy enforcement and telemetry work in a structured, test-first way.

## One-Sentence Objective

Package U1-B3 implementation knowledge into a high-yield learning module that makes the policy/telemetry bolt easier to understand, practice, and retain.

## Scope of This Version

This version adds and updates learning-oriented markdown artifacts:

- new study guide for U1-B3,
- README learning references and version marker,
- changelog entry and deletion notes for generated artifacts.

No additional production Rust behavior changes were introduced in this version.

## Detailed Changes

## 1) New Study Guide for U1-B3

Added:

- `learn/u1-b3-inbound-policy-telemetry-study-guide.md`

What it teaches:

- why `opportunistic` and `require-tls` are separate policy modes,
- how deterministic SMTP mapping works (`503` vs `530`, plus `454`/`451` context),
- how to reason about session telemetry fields and assertions,
- how to use `inbound_policy_telemetry.rs` as executable documentation,
- practical drills, self-check criteria, and next-step progression.

Learning design intent:

- keep complexity manageable while preserving technical rigor,
- focus on test-driven understanding over passive reading,
- align to current project sequence (U1 complete, U2 next).

## 2) README Alignment

Updated:

- version marker to `v0.1.5`,
- `learn/` tree to include the new U1-B3 guide,
- quick-start learning references to include U1-B3,
- roadmap learning note to include all U1 study guides (B1/B2/B3).

Reason:

- keep onboarding and learning pathways synchronized with current repository state.

## 3) Changelog Update

Updated:

- `CHANGELOG.md` with `v0.1.5` entry covering the new learning module and README changes.
- `### For Deletion` list now includes currently present generated artifacts and temp directories for manual cleanup.

Reason:

- preserve traceability of documentation releases,
- make cleanup expectations explicit without deleting files automatically.

## Traceability Links

- Study guide:
  - `learn/u1-b3-inbound-policy-telemetry-study-guide.md`
- U1-B3 implementation references:
  - `verzola-proxy/src/inbound/mod.rs`
  - `verzola-proxy/tests/inbound_policy_telemetry.rs`
- U1-B3 docs references:
  - `docs/inbound-policy-telemetry.md`
  - `docs/adr/0003-u1-b3-inbound-policy-and-telemetry.md`
  - `docs/reviews/u1-b3-operational-readiness.md`
  - `docs/bolts/u1-b3-traceability.md`

## Validation Notes

Documentation consistency checks performed:

- README learning references include U1-B3 guide,
- changelog includes v0.1.5 entry and deletion notes,
- version docs file is placed under `docs/` using kebab-case naming.

## Practical Next Use

Recommended usage sequence for new contributors:

1. Read `learn/u1-b1-inbound-starttls-study-guide.md`.
2. Read `learn/u1-b2-streaming-forwarder-study-guide.md`.
3. Read `learn/u1-b3-inbound-policy-telemetry-study-guide.md`.
4. Move to Unit U2 implementation work.
