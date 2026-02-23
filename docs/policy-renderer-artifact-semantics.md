# Policy Renderer Artifact Semantics (`verzolactl render`)

This document defines the generated config artifact produced by Unit U3 / Bolt U3-B2.

## Command Surface

Render effective config from a validated policy file:

```powershell
python -m verzola_control render <policy-file.yaml> --environment dev --output -
```

Supported environments:

- `dev`
- `staging`
- `prod`

## Render Behavior

- Policy file is validated first (same strict/non-strict behavior as `verzolactl validate`).
- If validation fails, render exits with code `1` and diagnostics.
- If validation passes, render emits deterministic JSON:
  - sorted object keys,
  - sorted domain policy entries,
  - fixed environment profile defaults.

## Artifact Shape

Top-level fields:

- `schema_version`: currently sourced from policy schema version (`1`).
- `environment`: selected render environment profile.
- `capability_hints`: currently includes `dns_txt` object or `null`.
- `proxy`: effective proxy configuration object.

`proxy.inbound` fields:

- `bind_addr`: environment-derived listener bind address.
- `banner_host`: environment-derived SMTP banner host.
- `tls_policy`: inbound policy mode from source policy.
- `allow_plaintext`: inbound `allow_plaintext` from source policy.
- `advertise_starttls`: currently always `true` for generated configs.
- `max_line_len`: environment-derived line-length guardrail.

`proxy.outbound` fields:

- `bind_addr`: environment-derived listener bind address.
- `banner_host`: environment-derived SMTP banner host.
- `tls_policy`: outbound policy mode from source policy.
- `per_domain_tls_policies`: sorted list of per-domain policies with:
  - `domain`,
  - `mode`,
  - optional `on_mismatch`.
- `max_line_len`: environment-derived line-length guardrail.

## Environment Profiles

Current built-in profiles:

- `dev`:
  - inbound bind `127.0.0.1:2525`
  - outbound bind `127.0.0.1:10025`
  - banner host `localhost`
- `staging`:
  - inbound bind `0.0.0.0:2525`
  - outbound bind `0.0.0.0:10025`
  - banner host `staging.verzola.local`
- `prod`:
  - inbound bind `0.0.0.0:25`
  - outbound bind `127.0.0.1:10025`
  - banner host `mail.verzola.local`

## Determinism and Traceability Notes

- The renderer intentionally excludes wall-clock timestamps in the artifact payload to keep snapshots stable.
- Domain ordering is canonicalized so policy reordering does not produce noisy diff churn.
- The output artifact can be committed or diffed as evidence for policy change review workflows.
