//! Query structs for the routes that take parameters. Field names are the
//! OpenAPI parameter names; `None` means "not sent".
//!
//! These are plain structs on purpose (no `#[non_exhaustive]`), so a caller
//! can write `MatchFilter { min_duration_s: Some(600), ..Default::default() }`.

use crate::{AccountId, HeroId};

/// Filters of `GET /v1/players/rank/distribution`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MatchFilter {
    /// Only matches started at or after this unix time.
    pub min_unix_timestamp: Option<u64>,
    /// Only matches started at or before this unix time.
    pub max_unix_timestamp: Option<u64>,
    /// Only matches at least this long, in seconds.
    pub min_duration_s: Option<u32>,
    /// Only matches at most this long, in seconds.
    pub max_duration_s: Option<u32>,
    /// Only matches in (or not in) the high-skill parties range.
    pub is_high_skill_range_parties: Option<bool>,
    /// Only matches in (or not in) the low-priority pool.
    pub is_low_pri_pool: Option<bool>,
    /// Only matches in (or not in) the new-player pool.
    pub is_new_player_pool: Option<bool>,
    /// Only matches with an id at or above this.
    pub min_match_id: Option<u64>,
    /// Only matches with an id at or below this.
    pub max_match_id: Option<u64>,
}

impl MatchFilter {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "min_unix_timestamp", self.min_unix_timestamp);
        push(&mut q, "max_unix_timestamp", self.max_unix_timestamp);
        push(&mut q, "min_duration_s", self.min_duration_s);
        push(&mut q, "max_duration_s", self.max_duration_s);
        push(
            &mut q,
            "is_high_skill_range_parties",
            self.is_high_skill_range_parties,
        );
        push(&mut q, "is_low_pri_pool", self.is_low_pri_pool);
        push(&mut q, "is_new_player_pool", self.is_new_player_pool);
        push(&mut q, "min_match_id", self.min_match_id);
        push(&mut q, "max_match_id", self.max_match_id);
        q
    }
}

/// Filters of `GET /v1/matches/active`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActiveMatchesQuery {
    /// Only matches with this player.
    pub account_id: Option<AccountId>,
    /// Only matches with any of these players.
    pub account_ids: Vec<AccountId>,
}

impl ActiveMatchesQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "account_id", self.account_id);
        if !self.account_ids.is_empty() {
            q.push(("account_ids", join_ids(&self.account_ids)));
        }
        q
    }
}

/// Options of `GET /v1/players/steam-search` beyond the search string.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SteamSearchQuery {
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Only players with at least this many matches in the last 30 days.
    pub min_matches_played_last_30d: Option<u32>,
    /// Only players whose last team averaged at least this badge.
    pub min_last_team_avg_badge: Option<u32>,
    /// How much recent activity weighs in the ranking of results.
    pub matches_played_weight: Option<f64>,
}

impl SteamSearchQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "limit", self.limit);
        push(
            &mut q,
            "min_matches_played_last_30d",
            self.min_matches_played_last_30d,
        );
        push(
            &mut q,
            "min_last_team_avg_badge",
            self.min_last_team_avg_badge,
        );
        push(&mut q, "matches_played_weight", self.matches_played_weight);
        q
    }
}

/// Filters of `GET /v1/players/hero-stats` beyond the account ids.
///
/// `game_mode` and `match_mode` are the server's strings (`normal`,
/// `street_brawl`, `explore_n_y_c`, `internal` for the game mode in the
/// OpenAPI document; `game_mode` defaults to `normal` server-side).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HeroStatsQuery {
    /// Only matches of this game mode.
    pub game_mode: Option<String>,
    /// Only matches of this match mode.
    pub match_mode: Option<String>,
    /// Only these heroes.
    pub hero_ids: Vec<HeroId>,
    /// Only matches started at or after this unix time.
    pub min_unix_timestamp: Option<u64>,
    /// Only matches started at or before this unix time.
    pub max_unix_timestamp: Option<u64>,
    /// Only matches at least this long, in seconds (server maximum 7000).
    pub min_duration_s: Option<u32>,
    /// Only matches at most this long, in seconds (server maximum 7000).
    pub max_duration_s: Option<u32>,
    /// Only matches where the player ended with at least this net worth.
    pub min_networth: Option<u64>,
    /// Only matches where the player ended with at most this net worth.
    pub max_networth: Option<u64>,
    /// Only matches with at least this average badge (0–116).
    pub min_average_badge: Option<u32>,
    /// Only matches with at most this average badge (0–116).
    pub max_average_badge: Option<u32>,
    /// Only matches with an id at or above this.
    pub min_match_id: Option<u64>,
    /// Only matches with an id at or below this.
    pub max_match_id: Option<u64>,
}

impl HeroStatsQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "game_mode", self.game_mode.clone());
        push(&mut q, "match_mode", self.match_mode.clone());
        if !self.hero_ids.is_empty() {
            q.push((
                "hero_ids",
                self.hero_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        push(&mut q, "min_unix_timestamp", self.min_unix_timestamp);
        push(&mut q, "max_unix_timestamp", self.max_unix_timestamp);
        push(&mut q, "min_duration_s", self.min_duration_s);
        push(&mut q, "max_duration_s", self.max_duration_s);
        push(&mut q, "min_networth", self.min_networth);
        push(&mut q, "max_networth", self.max_networth);
        push(&mut q, "min_average_badge", self.min_average_badge);
        push(&mut q, "max_average_badge", self.max_average_badge);
        push(&mut q, "min_match_id", self.min_match_id);
        push(&mut q, "max_match_id", self.max_match_id);
        q
    }
}

/// Filters of `GET /v1/players/{id}/mate-stats`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MateStatsQuery {
    /// Only matches of this game mode (server strings, see [`HeroStatsQuery`]).
    pub game_mode: Option<String>,
    /// Only matches started at or after this unix time.
    pub min_unix_timestamp: Option<u64>,
    /// Only matches started at or before this unix time.
    pub max_unix_timestamp: Option<u64>,
    /// Only matches at least this long, in seconds.
    pub min_duration_s: Option<u32>,
    /// Only matches at most this long, in seconds.
    pub max_duration_s: Option<u32>,
    /// Only matches with an id at or above this.
    pub min_match_id: Option<u64>,
    /// Only matches with an id at or below this.
    pub max_match_id: Option<u64>,
    /// Only teammates with at least this many shared matches.
    pub min_matches_played: Option<u32>,
    /// Only teammates with at most this many shared matches.
    pub max_matches_played: Option<u32>,
    /// Only matches where the two were in the same party.
    pub same_party: Option<bool>,
}

impl MateStatsQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "game_mode", self.game_mode.clone());
        push(&mut q, "min_unix_timestamp", self.min_unix_timestamp);
        push(&mut q, "max_unix_timestamp", self.max_unix_timestamp);
        push(&mut q, "min_duration_s", self.min_duration_s);
        push(&mut q, "max_duration_s", self.max_duration_s);
        push(&mut q, "min_match_id", self.min_match_id);
        push(&mut q, "max_match_id", self.max_match_id);
        push(&mut q, "min_matches_played", self.min_matches_played);
        push(&mut q, "max_matches_played", self.max_matches_played);
        push(&mut q, "same_party", self.same_party);
        q
    }
}

/// Filters of `GET /v1/players/{id}/enemy-stats`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnemyStatsQuery {
    /// Only matches of this game mode (server strings, see [`HeroStatsQuery`]).
    pub game_mode: Option<String>,
    /// Only matches started at or after this unix time.
    pub min_unix_timestamp: Option<u64>,
    /// Only matches started at or before this unix time.
    pub max_unix_timestamp: Option<u64>,
    /// Only matches at least this long, in seconds.
    pub min_duration_s: Option<u32>,
    /// Only matches at most this long, in seconds.
    pub max_duration_s: Option<u32>,
    /// Only matches with an id at or above this.
    pub min_match_id: Option<u64>,
    /// Only matches with an id at or below this.
    pub max_match_id: Option<u64>,
    /// Only opponents with at least this many shared matches.
    pub min_matches_played: Option<u32>,
    /// Only opponents with at most this many shared matches.
    pub max_matches_played: Option<u32>,
}

impl EnemyStatsQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        push(&mut q, "game_mode", self.game_mode.clone());
        push(&mut q, "min_unix_timestamp", self.min_unix_timestamp);
        push(&mut q, "max_unix_timestamp", self.max_unix_timestamp);
        push(&mut q, "min_duration_s", self.min_duration_s);
        push(&mut q, "max_duration_s", self.max_duration_s);
        push(&mut q, "min_match_id", self.min_match_id);
        push(&mut q, "max_match_id", self.max_match_id);
        push(&mut q, "min_matches_played", self.min_matches_played);
        push(&mut q, "max_matches_played", self.max_matches_played);
        q
    }
}

/// The server's list form for id parameters: comma-joined.
pub(crate) fn join_ids(ids: &[AccountId]) -> String {
    ids.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn push<T: ToString>(q: &mut Vec<(&'static str, String)>, name: &'static str, value: Option<T>) {
    if let Some(v) = value {
        q.push((name, v.to_string()));
    }
}
