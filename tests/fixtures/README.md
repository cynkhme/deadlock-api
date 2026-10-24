# Fixtures

Trimmed real responses from api.deadlock-api.com. Each file keeps the full
shape of one response and a few elements per list, so decoding tests see
every field without a megabyte of data.

| File | Request | Recorded | Trim |
|---|---|---|---|
| `match_metadata.json` | `GET /v1/matches/122539002/metadata` | 2026-10-24 | `jq -f scripts/trim-metadata.jq` on the full 0.97 MB response: 2 of 12 players, 3 `stats` samples, 5 `items`, 2 `death_details`, 2 paths with 10 positions, 1 damage dealer |
