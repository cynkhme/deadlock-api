//! Client for the [deadlock-api.com](https://deadlock-api.com) Deadlock match data API.
//!
//! Typed models for match metadata and the core match, player, Steam,
//! leaderboard and info endpoints; a blocking client (feature `blocking`,
//! ureq) and an async client (feature `async`, reqwest) over one sans-io
//! core; and a `Cache` trait so a rerun never re-fetches an immutable
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
//! - `fs-cache`: `FileCache`, a cache that stores one file per response.
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
mod error;
mod ids;

pub use cache::Cache;
pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use ids::{AccountId, HeroId, MatchId};
