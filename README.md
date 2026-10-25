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

The async client has the same methods; enable the `async` feature and add `.await`. It must run inside a tokio runtime, which your application starts (for example with `#[tokio::main]`, as `examples/async_match_summary.rs` does); outside one, the first request panics.

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
| `gzip` | no | gzip-compressed responses through both clients' built-in transports; adds `flate2`, and reqwest's decompression layer for the async client |

A client feature without a TLS feature is a compile error. For the async client alone: `cargo add deadlock-api --no-default-features --features async,rustls`.

With either TLS feature, both clients verify the server certificate against the operating system's trust store, so a CA installed on the machine (a corporate proxy, for example) is trusted. On Linux the store is the system CA bundle, or the file named by `SSL_CERT_FILE`. With `rustls` and a store that holds no certificate, the async client's `build()` fails with `Error::Transport`, and the blocking client fails every request with it.

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
| `heroes(&HeroesQuery)` | `GET /v1/assets/heroes` | `Vec<Hero>` |
| `hero(HeroId, &AssetsQuery)` | `GET /v1/assets/heroes/{id}` | `Hero` |
| `hero_by_name(&str, &AssetsQuery)` | `GET /v1/assets/heroes/by-name/{name}` | `Hero` |
| `items(&AssetsQuery)` | `GET /v1/assets/items` | `Vec<AssetItem>` |
| `item(ItemId, &AssetsQuery)` | `GET /v1/assets/items/{id}` | `AssetItem` |
| `item_by_class_name(&str, &AssetsQuery)` | `GET /v1/assets/items/{class_name}` | `AssetItem` |
| `hero_items(HeroId, &AssetsQuery)` | `GET /v1/assets/items/by-hero-id/{id}` | `Vec<AssetItem>` |
| `items_by_slot(ItemSlot, &AssetsQuery)` | `GET /v1/assets/items/by-slot-type/{slot}` | `Vec<AssetItem>` |
| `items_by_kind(ItemKind, &AssetsQuery)` | `GET /v1/assets/items/by-type/{kind}` | `Vec<AssetItem>` |
| `ranks(&AssetsQuery)` | `GET /v1/assets/ranks` | `Vec<Rank>` |
| `rank(u32, &AssetsQuery)` | `GET /v1/assets/ranks/{tier}` | `Rank` |
| `patches()` | `GET /v1/patches` | `Vec<Patch>` |
| `patch_feed()` | `GET /v2/patches` | `Vec<FeedItem>` |
| `patch_big_days()` | `GET /v1/patches/big-days` | `Vec<String>` |

A method that takes a list of ids returns an empty `Vec` for an empty list without sending a request. The asset routes are the game client's own data: `AssetItem` is an ability, an upgrade or a weapon, and an item of a type the crate does not know yet decodes as `AssetItem::Other` with its JSON kept whole. A hero or item name goes into the path percent-encoded; an empty name, `.`, `..` or a name with `/` is refused before sending. Model fields carry the server's JSON names unchanged, so the server's own documentation applies field for field.

Any other route goes through `get_json::<T>(path, query)` or `get_raw(path, query)`, which return the decoded body or the raw bytes and bypass the cache. The path starts with `/` and carries no `?` or `#`; query pairs go in `query`.

## Cache

`match_metadata` is the only cached route: a finished match does not change, and the server limits how often it fetches matches it has not ingested yet. Pass any `Cache` implementation to the builder, or `FileCache` with the `fs-cache` feature:

```rust,ignore
use deadlock_api::FileCache;

let api = Client::builder().cache(FileCache::new("cache/")).build()?;
```

`FileCache` writes `matches/{id}/metadata.json` under its directory, atomically. It never removes anything by itself; `prune_older_than` and `prune_to_size` remove entries when you call them. Give it a directory of its own: both treat every `*.json` file under it as a cached response. A failed cache read or write is logged through `log` and the request proceeds.

## Your own HTTP client

Each client sends its requests through a `Transport`: `blocking::Transport` for the blocking client, `Transport` for the async one. Pass your own to the builder with `.transport(...)` and ureq or reqwest is not used. The client still builds every request and keeps the cache, the retry policy, the status mapping and the decoding, so a transport that serves recorded responses runs your whole flow in a test without a network. The builder's timeouts, the `gzip` feature, the 64 MiB body limit and the redirect policy belong to the built-in transports and do not apply to a custom one, and its errors come back as `Error::Transport` and are never retried. The async client still needs a tokio runtime, with the time driver when it retries, and the `blocking` or `async` feature still compiles ureq or reqwest.

## Errors

Every method returns `deadlock_api::Result<T>`. A client retries only when its builder was given a `RetryPolicy` (see below). A 429 is `Error::RateLimited` and a 503 is `Error::Overloaded`; both carry the server's `Retry-After`, if it sent one:

```rust,ignore
match api.player_rank(AccountId(167576350)) {
    Err(e @ (Error::RateLimited { .. } | Error::Overloaded { .. })) => {
        let wait = e.retry_after().unwrap_or(Duration::from_secs(60));
        // wait, then try again
    }
    other => { /* ... */ }
}
```

A 404 is `Error::NotFound`, other statuses are `Error::Status` with the server's error message and the start of the body. The built-in transports do not follow redirects, so the API key never reaches another host.

With `.retry(RetryPolicy::new(3))` on the builder, a client sends a request up to three times in all. It retries a 503, a 429 whose `Retry-After` is at most a minute (`max_wait`), and a connection that failed before the request was sent. A 429 without `Retry-After` and a timeout after sending are returned at once, because the server may already have counted the request against its hourly limit. The pauses double from one second up to 30 seconds and never undercut the server's `Retry-After`.

## Tests

`cargo test` runs offline, against responses recorded from the live server in `tests/fixtures/`; every recorded success response must decode into its model and serialize back with all of its fields. The smoke tests in `tests/live.rs` request every typed route once on the live server, 33 requests at one per second, and run the same check on each response, so a field the server renames or retypes fails the test instead of decoding to `None`. They are ignored by default and fail without `DEADLOCK_API_LIVE=1`:

```sh
DEADLOCK_API_LIVE=1 cargo test --test live -- --ignored
```

## Minimum supported Rust version

Rust 1.98. Raising it is a minor-version change and is listed in the [changelog](CHANGELOG.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
