# Policy Schema Reference (`verzolactl validate`)

This document defines the strict schema currently enforced by Unit U3 / Bolt U3-B1.

## Supported File Types

- `.yaml`
- `.yml`
- `.toml`

## Top-Level Fields

- `version` (required): integer, currently `1`.
- `listeners` (required): object containing `inbound` and `outbound`.
- `domains` (optional): object keyed by domain name.
- `capability_hints` (optional): object for advisory capability metadata.

Unknown top-level fields are rejected in strict mode.

## `listeners`

Required object fields:

- `inbound`
- `outbound`

Each listener object requires:

- `mode`: one of:
  - `opportunistic`
  - `require-tls`
  - `require-pq`
- `allow_plaintext`: boolean

Unknown listener fields are rejected in strict mode.

## `domains`

Optional object where each key is a domain name (`example.com`) and each value is:

- `mode` (required): one of `opportunistic`, `require-tls`, `require-pq`
- `on_mismatch` (optional): one of `defer`, `reject`

Rules:

- Domain keys are normalized to lowercase (with trailing dots removed).
- Duplicate rules after normalization are rejected.
- `on_mismatch` is only valid when `mode = require-pq`.
- For `require-pq`, omitted `on_mismatch` defaults to `defer`.

## `capability_hints`

Optional object.

Supported fields:

- `dns_txt` (optional object):
  - `enabled` (required): boolean
  - `label` (optional): string, default `_verzola._tcp`

Rules:

- Unknown fields under `capability_hints` or `dns_txt` are rejected in strict mode.
- `dns_txt.label` must be non-empty and start with `_`.

## Validation Output

`verzolactl validate <policy-file>` returns:

- exit code `0` when valid,
- exit code `1` when parse/schema errors are found.

Diagnostics include:

- file path,
- field path,
- message,
- suggested correction.
