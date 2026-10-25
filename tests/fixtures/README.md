# Fixtures

Trimmed real responses from api.deadlock-api.com, recorded 2026-10-24 by
`scripts/record-fixtures.sh` (match 122539002, account 167576350, region
Europe, hero 2) unless noted. Each file keeps the full shape of one response
and a few elements per list, so decoding tests see every field without a
megabyte of data. The scripts named here are in the `scripts/` directory of
the repository, which the published crate does not include.

The responses are data served by deadlock-api.com, included here only as
test input. The crate's MIT and Apache-2.0 licences cover its code, not
this data.

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
| `assets_heroes.json` | `GET /v1/assets/heroes` (2026-10-25, full 1.9 MB response) | `scripts/trim-assets.sh`: the selectable hero with the most keys, a selectable hero without `gender`, a hero that cannot be selected |
| `assets_items.json` | `GET /v1/assets/items` (2026-10-25, full 6.4 MB response) | `scripts/trim-assets.sh`: per type, the fewest items that together carry every key any item of that type has (2 abilities, 2 upgrades, 3 weapons) |
| `assets_ranks.json` | `GET /v1/assets/ranks` (2026-10-25) | `scripts/trim-assets.sh`: tiers 0 and 1 |
| `patches.json` | `GET /v1/patches` (2026-10-25) | `scripts/trim-assets.sh`: first 2, `content_encoded` cut to 200 characters |
| `patch_feed.json` | `GET /v2/patches` (2026-10-25) | `scripts/trim-assets.sh`: the first forum post and the first Steam post, `content` cut to 200 characters |
| `patch_big_days.json` | `GET /v1/patches/big-days` (2026-10-25) | none |

Facts measured while recording (2026-10-24): `account_ids=1,2` and
`account_ids=1%2C2` return identical bodies; every account-like id in these
responses fits `u32`; `/v1/leaderboard/europe` answers 400 (`unknown variant
europe, expected one of Europe, Asia, NAmerica, SAmerica, Oceania`); no 429
or 503 was seen.

The asset and patch responses were recorded 2026-10-25 in one session of 9
requests, one per second, all 200 (the six listed above plus
`/v1/assets/heroes/1`, `/v1/assets/items/1548066885` and
`/v1/assets/ranks/1`, which only fed the drift check); every item of the
full `/v1/assets/items` response decoded into a known type.

Live smoke run (2026-10-24,
`DEADLOCK_API_LIVE=1 cargo test --test live -- --ignored`): 18 of 18 passed
in 25 s, 19 requests, no 429 or 503; the shape guard found no drift on any
of the 17 routes it checks, on the full 12-player metadata response or on
the metadata of a recently fetched match.

Live smoke run (2026-10-25, 15:54:55Z to 15:55:34Z, same command): 32 of 32
passed in 39 s, 33 requests, no 429 or 503. The 14 Assets and Patches tests
went through their typed methods, so the server accepted
`/v1/assets/heroes/by-name/Mo%20%26%20Krill`; the shape guard found no drift
on any route, including the three item routes without a recorded fixture
(by hero, by slot, by type). One more request in the same session,
`GET /v1/info/health` with `Accept-Encoding: gzip`, came back with
`content-encoding: gzip`, so the `gzip` feature takes effect against this
server.
