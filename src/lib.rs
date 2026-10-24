//! Client for the [deadlock-api.com](https://deadlock-api.com) Deadlock match data API.
//!
//! Typed models for match metadata and the core match, player, Steam,
//! leaderboard and info endpoints; a blocking client (feature `blocking`,
//! ureq) and an async client (feature `async`, reqwest) over one sans-io
//! core; and a [`Cache`] trait so a rerun never re-fetches an immutable
//! match. For the complete generated surface of the API see the official
//! [`deadlock-api/openapi-clients`](https://github.com/deadlock-api/openapi-clients).
//!
//! # Features
//!
//! - `blocking` (default): the synchronous `blocking::Client` on ureq.
//! - `async`: the asynchronous `Client` on reqwest; pulls in tokio.
//! - `rustls` (default) / `native-tls`: the TLS stack for both clients. With
//!   both enabled `rustls` wins; disable default features to use `native-tls`
//!   alone.
//! - `fs-cache`: [`FileCache`], a cache that stores one file per response.
//!
//! # Minimum supported Rust version
//!
//! Rust 1.85 (edition 2024). An MSRV bump is a minor-version change and is
//! listed in the changelog.
//!
//! # License
//!
//! MIT.
//!
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[macro_use]
mod config;
mod cache;
mod endpoint;
mod endpoints;
mod error;
mod ids;
pub mod types;

pub use cache::Cache;
#[cfg(feature = "fs-cache")]
pub use cache::FileCache;
pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use ids::{AccountId, HeroId, MatchId};
pub use types::filters::{ActiveMatchesQuery, MatchFilter, SteamSearchQuery};
pub use types::info::{ApiInfo, Health, TableSize};
pub use types::leaderboard::{Leaderboard, LeaderboardEntry, Region};
pub use types::match_metadata::{
    DamageDealer, DamageMatrix, Death, Item, MatchInfo, MatchMetadata, MatchPaths, MidBoss,
    Objective, Pause, Player, PlayerPath, PlayerStats, Pos, Team,
};
pub use types::matches::{
    ActiveMatch, ActiveMatchPlayer, MatchSalts, RecentMatch, RecentMatchPlayer, SpectateUrl,
};
pub use types::players::{
    AccountRank, EnemyStats, HeroStats, MatchHistoryEntry, MateStats, RankDistributionEntry,
    RankResponse,
};
pub use types::steam::{SteamFriend, SteamProfile};
