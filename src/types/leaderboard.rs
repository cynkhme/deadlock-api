//! Leaderboard routes and the region they are keyed by.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{AccountId, HeroId};

/// A leaderboard region, the path segment of `GET /v1/leaderboard/{region}`.
///
/// The server matches it exactly (`europe` is rejected). A value it may add
/// later arrives as [`Region::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
pub enum Region {
    /// `Europe`.
    Europe,
    /// `Asia`.
    Asia,
    /// `NAmerica`.
    NAmerica,
    /// `SAmerica`.
    SAmerica,
    /// `Oceania`.
    Oceania,
    /// A region this crate does not know, kept verbatim.
    Unknown(String),
}

impl Region {
    /// The value as the server spells it in the path.
    pub fn as_path_segment(&self) -> &str {
        match self {
            Region::Europe => "Europe",
            Region::Asia => "Asia",
            Region::NAmerica => "NAmerica",
            Region::SAmerica => "SAmerica",
            Region::Oceania => "Oceania",
            Region::Unknown(s) => s,
        }
    }
}

impl From<String> for Region {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Europe" => Region::Europe,
            "Asia" => Region::Asia,
            "NAmerica" => Region::NAmerica,
            "SAmerica" => Region::SAmerica,
            "Oceania" => Region::Oceania,
            _ => Region::Unknown(s),
        }
    }
}

impl From<Region> for String {
    fn from(r: Region) -> Self {
        r.as_path_segment().to_owned()
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_path_segment())
    }
}

/// `GET /v1/leaderboard/{region}` and `GET /v1/leaderboard/{region}/{hero_id}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Leaderboard {
    /// The entries, best first.
    #[serde(default)]
    pub entries: Vec<LeaderboardEntry>,
}

/// One row of a [`Leaderboard`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LeaderboardEntry {
    /// The displayed account name.
    #[serde(default)]
    pub account_name: Option<String>,
    /// Account ids the server matched to the name; may be empty.
    #[serde(default)]
    pub possible_account_ids: Vec<AccountId>,
    /// The position, starting at 1.
    #[serde(default)]
    pub rank: Option<u32>,
    /// The heroes the player ranks with.
    #[serde(default)]
    pub top_hero_ids: Vec<HeroId>,
}
