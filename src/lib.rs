//! Unofficial client for the [deadlock-api.com](https://deadlock-api.com) Deadlock match data API,
//! not affiliated with deadlock-api.com or Valve.
//!
//! Typed models for match metadata and the core match, player, Steam,
//! leaderboard and info endpoints; a blocking client (feature `blocking`,
//! ureq) and an async client (feature `async`, reqwest) over one sans-io
//! core; and a [`Cache`] trait so a rerun never re-fetches an immutable
//! match. For the complete generated surface of the API see the official
//! [`deadlock-api/openapi-clients`](https://github.com/deadlock-api/openapi-clients).
//!
//! # Quick start
//!
//! ```toml
//! [dependencies]
//! deadlock-api = "0.1"
//! ```
//!
//! ```no_run
//! # #[cfg(feature = "blocking")]
//! # fn main() -> deadlock_api::Result<()> {
//! use deadlock_api::blocking::Client;
//! use deadlock_api::{AccountId, Error, MatchId};
//!
//! let api = Client::builder()
//!     .api_key(std::env::var("DEADLOCK_API_KEY").ok())
//!     .build()?;
//!
//! let meta = api.match_metadata(MatchId(122539002))?;
//! let ids: Vec<AccountId> = meta.match_info.players.iter().map(|p| p.account_id).collect();
//! let profiles = api.steam_profiles(&ids)?;
//! for p in &profiles {
//!     println!("{}: {}", p.account_id, p.personaname.as_deref().unwrap_or("?"));
//! }
//!
//! match api.player_rank(AccountId(167576350)) {
//!     Err(e @ (Error::RateLimited { .. } | Error::Overloaded { .. })) => {
//!         let wait = e.retry_after().unwrap_or(std::time::Duration::from_secs(60));
//!         eprintln!("try again in {wait:?}");
//!     }
//!     other => {
//!         let rank = other?;
//!         println!("badge {:?}", rank.badge);
//!     }
//! }
//! # Ok(()) }
//! # #[cfg(not(feature = "blocking"))]
//! # fn main() {}
//! ```
//!
//! # Models
//!
//! Every model and query struct is at the crate root. Model fields carry the
//! server's JSON names unchanged, so the server's documentation applies field
//! for field; how [`MatchMetadata`] tolerates server drift is documented on
//! it. Query struct fields are the OpenAPI parameter names, `None` means the
//! parameter is not sent, and the structs are plain, so
//! `MatchFilter { min_duration_s: Some(600), ..Default::default() }` works.
//!
//! # Features
//!
//! - `blocking` (default): the synchronous [`blocking::Client`] on ureq.
//! - `async`: the asynchronous [`Client`] on reqwest; pulls in tokio.
//! - `rustls` (default) / `native-tls`: the TLS stack for both clients. With
//!   both enabled `rustls` wins; disable default features to use `native-tls`
//!   alone. Either way both clients verify certificates against the
//!   operating system's trust store (on Linux the system CA bundle, or the
//!   file named by `SSL_CERT_FILE`).
//! - `fs-cache`: [`FileCache`], a cache that stores one file per response.
//!
//! # Minimum supported Rust version
//!
//! Rust 1.98. An MSRV bump is a minor-version change and is listed in the
//! changelog.
//!
//! # License
//!
//! MIT OR Apache-2.0, at your option.
//!
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(
    not(any(feature = "blocking", feature = "async")),
    allow(
        dead_code,
        unused_imports,
        unused_macros,
        reason = "without a client feature only the models are used"
    )
)]

#[macro_use]
mod config;
#[cfg(feature = "blocking")]
pub mod blocking;
mod cache;
#[cfg(feature = "async")]
mod client;
mod endpoint;
mod endpoints;
mod error;
mod ids;
mod types;

pub use cache::Cache;
#[cfg(feature = "fs-cache")]
pub use cache::FileCache;
#[cfg(feature = "async")]
pub use client::{Client, ClientBuilder};
pub use config::Config;
pub(crate) use config::ConfigBuilder;
pub use error::{Error, Result};
pub use ids::{AccountId, HeroId, MatchId};
pub use types::filters::{
    ActiveMatchesQuery, EnemyStatsQuery, HeroStatsQuery, MatchFilter, MateStatsQuery,
    SteamSearchQuery,
};
pub use types::info::{ApiInfo, Health, TableSize};
pub use types::leaderboard::{Leaderboard, LeaderboardEntry, Region};
pub use types::match_metadata::{
    AbilityStat, DamageDealer, DamageMatrix, Death, Item, MatchInfo, MatchMetadata, MatchPaths,
    MidBoss, Objective, Pause, Player, PlayerPath, PlayerStats, Pos, Team,
};
pub use types::matches::{
    ActiveMatch, ActiveMatchPlayer, MatchSalts, RecentMatch, RecentMatchPlayer, SpectateUrl,
};
pub use types::players::{
    AccountRank, EnemyStats, HeroStats, MatchHistoryEntry, MateStats, RankDistributionEntry,
    RankResponse,
};
pub use types::steam::{SteamFriend, SteamProfile};

#[cfg(all(doctest, feature = "blocking"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
