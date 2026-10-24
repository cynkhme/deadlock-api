# Fixtures

Trimmed real responses from api.deadlock-api.com, recorded 2026-10-24 by
`scripts/record-fixtures.sh` (match 122539002, account 167576350, region
Europe, hero 2) unless noted. Each file keeps the full shape of one response
and a few elements per list, so decoding tests see every field without a
megabyte of data.

| File | Request | Trim |
|---|---|---|
| `match_metadata.json` | `GET /v1/matches/122539002/metadata` (full 0.97 MB response, recorded in the same session) | `jq -f scripts/trim-metadata.jq`: players of slots 10 and 4 (they carry `mvp_rank` and `ability_stats`), 3 `stats` samples, 5 `items`, 2 `death_details`, 2 paths with 10 positions, 1 damage dealer |
| `match_salts.json` | `GET /v1/matches/122539002/salts` | none |
| `spectate_url_400.json` | `GET /v1/matches/122539002/live/url` → **400** `{"status":400,"error":"Match 122539002 cannot be live"}` | none; this is the finished-match case |
| `active_matches.json` | `GET /v1/matches/active` | first 3 |
| `recently_fetched.json` | `GET /v1/matches/recently-fetched` | first 3 |
| `match_history.json` | `GET /v1/players/167576350/match-history` | first 3 |
| `player_rank.json` | `GET /v1/players/167576350/rank` | none |
| `player_ranks.json` | `GET /v1/players/rank?account_ids=167576350,3226752420` | none |
| `rank_distribution.json` | `GET /v1/players/rank/distribution` | first 5 |
| `hero_stats.json` | `GET /v1/players/hero-stats?account_ids=167576350` | first 3 |
| `mate_stats.json` | `GET /v1/players/167576350/mate-stats` | first 3, `matches` to 5 |
| `enemy_stats.json` | `GET /v1/players/167576350/enemy-stats` | first 3, `matches` to 5 |
| `steam_profiles.json` | `GET /v1/players/steam?account_ids=167576350,3226752420` | `friends` to 2, then `jq -f scripts/pseudonymise-steam.jq`: names, profile and avatar URLs, country and friend ids replaced by stand-ins that resolve to no one |
| `steam_search.json` | `GET /v1/players/steam-search?search_query=recrent&limit=3` | first 1, `friends` to 2, then `scripts/pseudonymise-steam.jq`, which also replaces the account ids of these strangers |
| `leaderboard.json` | `GET /v1/leaderboard/Europe` | `entries` to 3, names replaced by `player-<rank>` |
| `hero_leaderboard.json` | `GET /v1/leaderboard/Europe/2` | `entries` to 3, names replaced by `player-<rank>` |
| `info.json` | `GET /v1/info` | none |
| `health.json` | `GET /v1/info/health` | none |

Facts measured while recording (2026-10-24): `account_ids=1,2` and
`account_ids=1%2C2` return identical bodies; every account-like id in these
responses fits `u32`; `/v1/leaderboard/europe` answers 400 (`unknown variant
europe, expected one of Europe, Asia, NAmerica, SAmerica, Oceania`); no 429
or 503 was seen.

Live smoke run (2026-10-24,
`DEADLOCK_API_LIVE=1 cargo test --test live -- --ignored`): 18 of 18 passed
in 22 s, 19 requests, no 429 or 503; the shape guard found no drift on the
full 12-player metadata response.
