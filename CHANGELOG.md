# Changelog

All notable changes to this crate are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html): while it is at 0.x, a breaking change raises the minor version.

## [Unreleased]

### Added

- Typed match metadata (`MatchMetadata`), decoded tolerantly: beyond the few fields a match cannot lack, a missing or `null` field is `None` or an empty list, and a malformed sub-object (paths, damage matrix) or element of the larger lists (stats, items, deaths, abilities, objectives, mid boss, pauses, teams) is dropped with a `log` warning instead of failing the match.
- 18 typed endpoints for matches, players, Steam profiles, leaderboards and server info.
- `blocking::Client` on ureq (feature `blocking`, default) and `Client` on reqwest (feature `async`), with the same methods.
- `get_json` and `get_raw` for any other route.
- The `Cache` trait and `FileCache` (feature `fs-cache`); only match metadata is cached.
- `Error` with `retry_after()` for 429 and 503 responses; no automatic retries.
- TLS through `rustls` (default) or `native-tls`.
