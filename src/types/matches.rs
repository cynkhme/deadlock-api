//! Match routes other than metadata: salts, the spectate URL, active and
//! recently fetched matches.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AccountId, HeroId};

/// `GET /v1/matches/{id}/salts`: where the match's replay and metadata live.
///
/// Not cached: `replay_salt` is `null` until the replay is available.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MatchSalts {
    /// The match id.
    pub match_id: u64,
    /// The replay cluster the files are served from.
    #[serde(default)]
    pub cluster_id: Option<u32>,
    /// Salt of the metadata file.
    #[serde(default)]
    pub metadata_salt: Option<u64>,
    /// Salt of the replay; `null` until the replay exists.
    #[serde(default)]
    pub replay_salt: Option<u64>,
    /// Full URL of the metadata file.
    #[serde(default)]
    pub metadata_url: Option<String>,
    /// Full URL of the replay; `null` until the replay exists.
    #[serde(default)]
    pub demo_url: Option<String>,
}

/// `GET /v1/matches/{id}/live/url` for a match that is live.
///
/// Typed from the OpenAPI document (`MatchSpectateResponse`): no live
/// match was available when the fixtures were recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpectateUrl {
    /// The match id.
    pub match_id: u64,
    /// The broadcast URL to spectate from.
    pub broadcast_url: String,
    /// The lobby id.
    #[serde(default)]
    pub lobby_id: Option<u64>,
    /// Unix seconds when the broadcast started.
    #[serde(default)]
    pub started_at: Option<u64>,
    /// Unix seconds when the entry was last updated.
    #[serde(default)]
    pub updated_at: Option<u64>,
}

/// One entry of `GET /v1/matches/active`.
///
/// The server sends several values twice, as a number and as a parsed
/// string (`match_mode` and `match_mode_parsed`); both are kept, as the
/// field names mirror the server one to one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActiveMatch {
    /// The match id.
    pub match_id: u64,
    /// Unix seconds when the match started.
    #[serde(default)]
    pub start_time: Option<u64>,
    /// Seconds played so far; `null` in the sample.
    #[serde(default)]
    pub duration_s: Option<u32>,
    /// The winning team, `null` while the match runs.
    #[serde(default)]
    pub winning_team: Option<u32>,
    /// `winning_team` as the server names it.
    #[serde(default)]
    pub winning_team_parsed: Option<String>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub match_mode: Option<u32>,
    /// `match_mode` as the server names it, e.g. `Unranked`.
    #[serde(default)]
    pub match_mode_parsed: Option<String>,
    /// Observed values: `1` and `4`. Undocumented upstream.
    #[serde(default)]
    pub game_mode: Option<u32>,
    /// `game_mode` as the server names it, e.g. `KECitadelGameModeStreetBrawl`.
    #[serde(default)]
    pub game_mode_parsed: Option<String>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub region_mode: Option<u32>,
    /// `region_mode` as the server names it, e.g. `europe`.
    #[serde(default)]
    pub region_mode_parsed: Option<String>,
    /// The players in the match.
    #[serde(default)]
    pub players: Vec<ActiveMatchPlayer>,
    /// The Steam lobby id; exceeds `u32`.
    #[serde(default)]
    pub lobby_id: Option<u64>,
    /// Version of the game mode.
    #[serde(default)]
    pub game_mode_version: Option<u32>,
    /// Net worth of team 0.
    #[serde(default)]
    pub net_worth_team_0: Option<u64>,
    /// Net worth of team 1.
    #[serde(default)]
    pub net_worth_team_1: Option<u64>,
    /// Current spectators.
    #[serde(default)]
    pub spectators: Option<u32>,
    /// Free spectator slots.
    #[serde(default)]
    pub open_spectator_slots: Option<u32>,
    /// Bit mask of objectives team 0 destroyed.
    #[serde(default)]
    pub objectives_mask_team0: Option<u64>,
    /// Bit mask of objectives team 1 destroyed.
    #[serde(default)]
    pub objectives_mask_team1: Option<u64>,
    /// Matchmaking score of the lobby.
    #[serde(default)]
    pub match_score: Option<u32>,
    /// Client compatibility version.
    #[serde(default)]
    pub compat_version: Option<u32>,
    /// Whether the match is in the low-priority pool.
    #[serde(default)]
    pub low_pri_pool: Option<bool>,
    /// Per-team entries (`team`, `team_parsed`, `net_worth`, `objectives_mask`, `brawl_score`); shape not pinned.
    #[serde(default)]
    pub team_stats: Vec<Value>,
}

/// A player in an [`ActiveMatch`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActiveMatchPlayer {
    /// The player's Steam account id.
    pub account_id: AccountId,
    /// The team index.
    #[serde(default)]
    pub team: Option<u32>,
    /// `team` as the server names it, e.g. `Team0`.
    #[serde(default)]
    pub team_parsed: Option<String>,
    /// The hero played.
    #[serde(default)]
    pub hero_id: Option<HeroId>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub abandoned: Option<Value>,
}

/// One entry of `GET /v1/matches/recently-fetched`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RecentMatch {
    /// The match id.
    pub match_id: u64,
    /// Unix seconds when the match started.
    #[serde(default)]
    pub start_time: Option<u64>,
    /// Match length in seconds.
    #[serde(default)]
    pub duration_s: Option<u32>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub match_mode: Option<u32>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub game_mode: Option<u32>,
    /// Average rank badge of team 0.
    #[serde(default)]
    pub average_badge_team0: Option<u32>,
    /// Average rank badge of team 1.
    #[serde(default)]
    pub average_badge_team1: Option<u32>,
    /// Average rank badge of the match; `null` in the sample.
    #[serde(default)]
    pub average_badge: Option<u32>,
    /// The players, by account id only.
    #[serde(default)]
    pub players: Vec<RecentMatchPlayer>,
}

/// A player in a [`RecentMatch`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RecentMatchPlayer {
    /// The player's Steam account id.
    pub account_id: AccountId,
}
