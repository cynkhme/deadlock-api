# Changelog

All notable changes to this crate are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html): while it is at 0.x, a breaking change raises the minor version.

## [Unreleased]

## [0.1.2] - 2026-10-25

### Fixed

- The `RetryPolicy` documentation said every failed name lookup is retried. The blocking client retries one only when it timed out or found no address, not when the system resolver failed; the async client retries a failed lookup.

## [0.1.1] - 2026-10-25

### Added

- The `gzip` feature, off by default: the built-in transports of both clients ask for gzip-compressed responses and decompress them.
- `FileCache::prune_older_than` and `FileCache::prune_to_size`, which remove cached responses by write time and report what they removed as `Pruned`.
- `blocking::Transport` and `Transport` (with `TransportFuture` and `BoxError`), and `transport` on both client builders: a client can send its requests through your own HTTP client and keeps the cache, the retry policy, the status mapping and the decoding. Both traits are implemented for `&T`, `Box<T>` and `Arc<T>`. `http` 1 is now a public dependency.
- Typed methods for the asset and patch routes on both clients: `heroes`, `hero`, `hero_by_name`, `items`, `item`, `item_by_class_name`, `hero_items`, `items_by_slot`, `items_by_kind`, `ranks`, `rank`, `patches`, `patch_feed` and `patch_big_days`, with the models `Hero`, `HeroDescription`, `AssetItem` (`AssetAbility`, `AssetUpgrade`, `AssetWeapon`, or `Other` for a type the crate does not know), `Rank`, `Patch`, `FeedItem`, `Guid` and `Category`, the queries `AssetsQuery` and `HeroesQuery`, the string enums `Language`, `ItemSlot` and `ItemKind`, and the id `ItemId`.
- The `Models` section of the crate documentation shows how a consumer's test builds a model from JSON, which fields `MatchMetadata` requires, and why such a test checks list lengths.
- `RetryPolicy` and `retry` on both client builders. Without a policy every request is sent once, as before; with one, a client retries a 503, a 429 with a `Retry-After` up to `max_wait`, and a connection that failed before the request was sent.

### Changed

- Both built-in transports read a response body of at most 64 MiB and return `Error::Transport` beyond it: the blocking client's limit was ureq's 10 MiB, the async client had none.

## [0.1.0] - 2026-10-25

### Added

- Typed match metadata (`MatchMetadata`), decoded tolerantly: beyond the few fields a match cannot lack, a missing or `null` field is `None` or an empty list, and a malformed sub-object (paths, damage matrix) or element of the larger lists (stats, items, deaths, abilities, objectives, mid boss, pauses, teams) is dropped with a `log` warning instead of failing the match.
- 18 typed endpoints for matches, players, Steam profiles, leaderboards and server info.
- `AccountId`, `HeroId` and `MatchId` newtypes; every model and query struct is at the crate root.
- `blocking::Client` on ureq (feature `blocking`, default) and `Client` on reqwest (feature `async`), with the same methods.
- `get_json` and `get_raw` for any other route. They refuse a query pair named `api_key`, and a path that does not start with `/` or carries `?` or `#`, so a request stays on the configured server.
- The API key is sent only as the `X-API-KEY` header, never in a URL. Redirects are not followed, so the key is never re-sent to another host.
- The `Cache` trait and `FileCache` (feature `fs-cache`); only match metadata is cached.
- `Error` with `retry_after()` for 429 and 503 responses; no automatic retries.
- TLS through `rustls` (default) or `native-tls`. Both clients verify certificates against the operating system's trust store (on Linux the system CA bundle, or the file named by `SSL_CERT_FILE`). With `rustls` and an empty store, the async client's `build()` and every blocking request fail with `Error::Transport`.
- Minimum supported Rust version 1.98.

[0.1.2]: https://github.com/cynkhme/deadlock-api/releases/tag/v0.1.2
[0.1.1]: https://github.com/cynkhme/deadlock-api/releases/tag/v0.1.1
[0.1.0]: https://github.com/cynkhme/deadlock-api/releases/tag/v0.1.0
