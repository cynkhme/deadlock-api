# deadlock-api

An unofficial Rust client for the Deadlock match data API at [deadlock-api.com](https://deadlock-api.com), not affiliated with deadlock-api.com or Valve. It gives typed match metadata, 18 typed endpoints for matches, players, Steam profiles, leaderboards and server info, a blocking and an async client, and a cache trait so an immutable match is fetched once.

For every endpoint of the API as generated code, see the official [`deadlock-api/openapi-clients`](https://github.com/deadlock-api/openapi-clients). This crate types the match metadata from real responses (the OpenAPI document describes it as a plain object), works without an async runtime, and leaves the rest of the API reachable through `get_json` and `get_raw`.

## Usage

```sh
cargo add deadlock-api
```

```rust,no_run
use deadlock_api::blocking::Client;
use deadlock_api::{AccountId, MatchId};

fn main() -> deadlock_api::Result<()> {
    let api = Client::builder()
        .api_key(std::env::var("DEADLOCK_API_KEY").ok())
        .build()?;

    let meta = api.match_metadata(MatchId(122539002))?;
    let ids: Vec<AccountId> = meta.match_info.players.iter().map(|p| p.account_id).collect();
    for profile in api.steam_profiles(&ids)? {
        println!("{}: {:?}", profile.account_id, profile.personaname);
    }
    Ok(())
}
```

The async client has the same methods; enable the `async` feature and add `.await`:

```rust,ignore
let api = deadlock_api::Client::builder().build()?;
let meta = api.match_metadata(MatchId(122539002)).await?;
```

Both examples in [`examples/`](examples) print a match summary:

```sh
cargo run --example match_summary -- 122539002
cargo run --example async_match_summary --features async -- 122539002
```

An API key is optional; when set, it is sent in the `X-API-KEY` header and never in the URL.

## Features

| Feature | Default | What it enables |
|---|---|---|
| `blocking` | yes | `deadlock_api::blocking::Client` on ureq |
| `rustls` | yes | rustls as the TLS stack of both clients |
| `async` | no | `deadlock_api::Client` on reqwest; pulls in tokio |
| `native-tls` | no | the platform TLS stack; with `rustls` also enabled, `rustls` wins |
| `fs-cache` | no | `FileCache`, one file per cached response |

A client feature without a TLS feature is a compile error. For the async client alone: `cargo add deadlock-api --no-default-features --features async,rustls`.

## Endpoints

| Method | Route | Returns |
|---|---|---|
| `match_metadata(MatchId)` | `GET /v1/matches/{id}/metadata` | `MatchMetadata` (cached) |
| `match_salts(MatchId)` | `GET /v1/matches/{id}/salts` | `MatchSalts` |
| `match_spectate_url(MatchId)` | `GET /v1/matches/{id}/live/url` | `Option<SpectateUrl>`, `None` for a finished match |
| `active_matches(&ActiveMatchesQuery)` | `GET /v1/matches/active` | `Vec<ActiveMatch>` |
| `recently_fetched_matches()` | `GET /v1/matches/recently-fetched` | `Vec<RecentMatch>` |
| `match_history(AccountId)` | `GET /v1/players/{id}/match-history` | `Vec<MatchHistoryEntry>` |
| `player_rank(AccountId)` | `GET /v1/players/{id}/rank` | `RankResponse` |
| `player_ranks(&[AccountId])` | `GET /v1/players/rank` | `Vec<AccountRank>` |
| `rank_distribution(&MatchFilter)` | `GET /v1/players/rank/distribution` | `Vec<RankDistributionEntry>` |
| `hero_stats(&[AccountId], &HeroStatsQuery)` | `GET /v1/players/hero-stats` | `Vec<HeroStats>` |
| `mate_stats(AccountId, &MateStatsQuery)` | `GET /v1/players/{id}/mate-stats` | `Vec<MateStats>` |
| `enemy_stats(AccountId, &EnemyStatsQuery)` | `GET /v1/players/{id}/enemy-stats` | `Vec<EnemyStats>` |
| `steam_profiles(&[AccountId])` | `GET /v1/players/steam` | `Vec<SteamProfile>` |
| `steam_search(&str, &SteamSearchQuery)` | `GET /v1/players/steam-search` | `Vec<SteamProfile>` |
| `leaderboard(Region)` | `GET /v1/leaderboard/{region}` | `Leaderboard` |
| `hero_leaderboard(Region, HeroId)` | `GET /v1/leaderboard/{region}/{hero_id}` | `Leaderboard` |
| `info()` | `GET /v1/info` | `ApiInfo` |
| `health()` | `GET /v1/info/health` | `Health` |

A method that takes a list of ids returns an empty `Vec` for an empty list without sending a request. Model fields carry the server's JSON names unchanged, so the server's own documentation applies field for field.

Any other route goes through `get_json::<T>(path, query)` or `get_raw(path, query)`, which return the decoded body or the raw bytes and bypass the cache.

## Cache

`match_metadata` is the only cached route: a finished match does not change, and the server limits how often it fetches matches it has not ingested yet. Pass any `Cache` implementation to the builder, or `FileCache` with the `fs-cache` feature:

```rust,ignore
use deadlock_api::FileCache;

let api = Client::builder().cache(FileCache::new("cache/")).build()?;
```

`FileCache` writes `matches/{id}/metadata.json` under its directory, atomically. It never prunes; the directory is yours to clean. A failed cache read or write is logged through `log` and the request proceeds.

## Errors

Every method returns `deadlock_api::Result<T>`. The crate does not retry. A 429 is `Error::RateLimited` and a 503 is `Error::Overloaded`; both carry the server's `Retry-After`, if it sent one:

```rust,ignore
match api.player_rank(AccountId(167576350)) {
    Err(e @ (Error::RateLimited { .. } | Error::Overloaded { .. })) => {
        let wait = e.retry_after().unwrap_or(Duration::from_secs(60));
        // wait, then try again
    }
    other => { /* ... */ }
}
```

A 404 is `Error::NotFound`, other statuses are `Error::Status` with the server's error message and the start of the body. Redirects are not followed, so the API key never reaches another host.

## Tests

`cargo test` runs offline, against responses recorded from the live server in `tests/fixtures/`. The smoke tests in `tests/live.rs` call every typed route once on the live server, one request per second, and report a change in the server's schema as a failed decode:

```sh
DEADLOCK_API_LIVE=1 cargo test --test live -- --ignored
```

## Minimum supported Rust version

Rust 1.98. Raising it is a minor-version change and is listed in the [changelog](CHANGELOG.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
