# Changelog

All notable changes to this crate are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html): while it is at 0.x, a breaking change raises the minor version.

## [0.1.0] - 2026-10-24

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

[0.1.0]: https://github.com/cynkhme/deadlock-api/releases/tag/v0.1.0
