//! `GET /v1/info` and `GET /v1/info/health`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// `GET /v1/info`: ingestion throughput and table sizes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApiInfo {
    /// Matches fetched per day.
    #[serde(default)]
    pub fetched_matches_per_day: Option<u64>,
    /// Matches users asked the server to ingest in the last 24 hours.
    #[serde(default)]
    pub user_ingested_matches_last24h: Option<u64>,
    /// Database tables by name.
    #[serde(default)]
    pub table_sizes: BTreeMap<String, TableSize>,
}

/// One table in [`ApiInfo::table_sizes`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TableSize {
    /// Whether the entry is a view.
    #[serde(default)]
    pub is_view: Option<bool>,
    /// Row count.
    #[serde(default)]
    pub rows: Option<u64>,
    /// Compressed size in bytes.
    #[serde(default)]
    pub data_compressed_bytes: Option<u64>,
    /// Uncompressed size in bytes.
    #[serde(default)]
    pub data_uncompressed_bytes: Option<u64>,
}

/// `GET /v1/info/health`: which backing services answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Health {
    /// Service name to whether it is healthy.
    #[serde(default)]
    pub services: BTreeMap<String, bool>,
}
