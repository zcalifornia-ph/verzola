# TLS Negotiation Classification Reference (U4-B1)

## Scope

Reference for `Unit U4 / Bolt U4-B1` negotiated-group metadata parsing and classification in `verzola-proxy/src/tls/mod.rs`.

This mapping is intentionally conservative for security-sensitive policy usage:

- unknown/unmapped group names classify as `none`,
- hybrid PQ groups classify as `pq`,
- ambiguous metadata must not silently count as PQ-capable.

## Classification Model

- `pq`:
  - negotiated group indicates post-quantum or hybrid PQ usage.
- `classical`:
  - negotiated group matches a known classical TLS named group.
- `none`:
  - no group metadata is present,
  - metadata is placeholder/unknown (`none`, `unknown`),
  - metadata cannot be safely mapped yet (for example, unmapped numeric IDs).

## Supported Metadata Parse Shapes

Current parser accepts common semi-structured forms:

- `negotiated_group=X25519`
- `group: secp256r1`
- `key_exchange_group = ffdhe3072`
- `"negotiated_group": "X25519MLKEM768"`
- `named_group X448`
- raw group token only: `X25519`

If the parser does not find a negotiated-group key (or a safe single-token value), the result is `none`.

## Mapping Rules (Current Bolt)

### `pq`

Classify as `pq` when normalized group name contains a known PQ/KEM marker, including hybrid names.

Current markers:

- `mlkem`
- `kyber`
- `frodo`
- `ntru`
- `saber`
- `bike`
- `hqc`
- `sntrup`
- `mceliece`
- `xwing`

Examples:

- `X25519MLKEM768` -> `pq`
- `x25519_kyber768draft00` -> `pq`

### `classical`

Classify as `classical` when normalized group name matches one of the current classical group names:

- `x25519`, `curve25519`
- `x448`, `curve448`
- `secp256r1`, `prime256v1`
- `secp384r1`
- `secp521r1`
- `p256`, `p384`, `p521`
- `brainpoolp256r1`, `brainpoolp384r1`, `brainpoolp512r1`
- `ffdhe2048`, `ffdhe3072`, `ffdhe4096`, `ffdhe6144`, `ffdhe8192`

Examples:

- `X25519` -> `classical`
- `secp256r1` -> `classical`
- `ffdhe3072` -> `classical`

### `none`

Classify as `none` when:

- group metadata is absent,
- parsed value is placeholder (`none`, `unknown`),
- parsed value is not yet mapped (for example `0x6399`).

Examples:

- `cipher=TLS_AES_128_GCM_SHA256` -> `none` (no negotiated-group field)
- `negotiated_group=0x6399` -> `none` (unmapped numeric ID)
- empty metadata -> `none`

## Limitations and Follow-Up

- Numeric IANA/draft group IDs are not mapped in U4-B1.
- Parser coverage is intentionally focused on common TLS adapter/log formats, not full JSON/log parsing.
- Later U4 bolts may expand metadata sources (for example DNS hints and strict `require-pq` policy handling), but should reuse this conservative classification contract unless a reviewed change is introduced.
