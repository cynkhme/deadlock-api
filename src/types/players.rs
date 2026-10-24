//! Player routes: match history, ranks, hero, mate and enemy statistics.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AccountId, HeroId, MatchId};

/// One entry of `GET /v1/players/{id}/match-history`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MatchHistoryEntry {
    /// The player's Steam account id.
    pub account_id: AccountId,
    /// The match id.
    pub match_id: MatchId,
    /// The hero played.
    #[serde(default)]
    pub hero_id: Option<HeroId>,
    /// Hero level at the end.
    #[serde(default)]
    pub hero_level: Option<u32>,
    /// Unix seconds when the match started.
    #[serde(default)]
    pub start_time: Option<u64>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub game_mode: Option<u32>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub match_mode: Option<u32>,
    /// The player's team index.
    #[serde(default)]
    pub player_team: Option<u32>,
    /// Kills.
    #[serde(default)]
    pub player_kills: Option<u32>,
    /// Deaths.
    #[serde(default)]
    pub player_deaths: Option<u32>,
    /// Assists.
    #[serde(default)]
    pub player_assists: Option<u32>,
    /// Denies.
    #[serde(default)]
    pub denies: Option<u32>,
    /// Net worth in souls at the end.
    #[serde(default)]
    pub net_worth: Option<u64>,
    /// Last hits.
    #[serde(default)]
    pub last_hits: Option<u32>,
    /// Seconds into the match the player abandoned it; `0` when they did not.
    #[serde(default)]
    pub abandoned_time_s: Option<u32>,
    /// Match length in seconds.
    #[serde(default)]
    pub match_duration_s: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub match_result: Option<u32>,
    /// Bit mask of objectives team 0 destroyed.
    #[serde(default)]
    pub objectives_mask_team0: Option<u64>,
    /// Bit mask of objectives team 1 destroyed.
    #[serde(default)]
    pub objectives_mask_team1: Option<u64>,
    /// Observed values: `2`. Undocumented upstream.
    #[serde(default)]
    pub player_match_outcome: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub ranked_calibration_match: Option<u32>,
    /// Whether the match was in the low-priority pool.
    #[serde(default)]
    pub low_pri_pool: Option<bool>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub brawl_score_team0: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub brawl_score_team1: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub brawl_avg_round_time_s: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub ranked_display_badge: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub ranked_delta: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub ranked_used_demotion_protection: Option<Value>,
}

/// `GET /v1/players/{id}/rank`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RankResponse {
    /// The rank badge, `0` when unranked.
    #[serde(default)]
    pub badge: Option<u32>,
    /// The rank tier.
    #[serde(default)]
    pub rank: Option<u32>,
    /// The sub-rank within the tier.
    #[serde(default)]
    pub subrank: Option<u32>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub last_match: Option<Value>,
}

/// One entry of `GET /v1/players/rank`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountRank {
    /// The player's Steam account id.
    pub account_id: AccountId,
    /// The rank badge, `0` when unranked.
    #[serde(default)]
    pub badge: Option<u32>,
    /// The rank tier.
    #[serde(default)]
    pub rank: Option<u32>,
    /// The sub-rank within the tier.
    #[serde(default)]
    pub subrank: Option<u32>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub last_match: Option<Value>,
}

/// One entry of `GET /v1/players/rank/distribution`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RankDistributionEntry {
    /// The rank badge.
    #[serde(default)]
    pub badge: Option<u32>,
    /// The rank tier.
    #[serde(default)]
    pub rank: Option<u32>,
    /// The sub-rank within the tier.
    #[serde(default)]
    pub subrank: Option<u32>,
    /// Players holding this badge.
    #[serde(default)]
    pub players: Option<u64>,
}

/// One entry of `GET /v1/players/hero-stats`: one player on one hero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HeroStats {
    /// The player's Steam account id.
    pub account_id: AccountId,
    /// The hero.
    pub hero_id: HeroId,
    /// Matches played on the hero.
    #[serde(default)]
    pub matches_played: Option<u32>,
    /// Unix seconds of the last match.
    #[serde(default)]
    pub last_played: Option<u64>,
    /// Seconds played on the hero.
    #[serde(default)]
    pub time_played: Option<u64>,
    /// Matches won.
    #[serde(default)]
    pub wins: Option<u32>,
    /// Average level at the end of a match.
    #[serde(default)]
    pub ending_level: Option<f64>,
    /// Total kills.
    #[serde(default)]
    pub kills: Option<u32>,
    /// Total deaths.
    #[serde(default)]
    pub deaths: Option<u32>,
    /// Total assists.
    #[serde(default)]
    pub assists: Option<u32>,
    /// Total damage to players.
    #[serde(default)]
    pub total_player_damage: Option<u64>,
    /// Total damage taken from players.
    #[serde(default)]
    pub total_player_damage_taken: Option<u64>,
    /// Total damage to bosses.
    #[serde(default)]
    pub total_boss_damage: Option<u64>,
    /// Total damage to lane creeps.
    #[serde(default)]
    pub total_creep_damage: Option<u64>,
    /// Total damage to neutral creeps.
    #[serde(default)]
    pub total_neutral_damage: Option<u64>,
    /// Denies per match.
    #[serde(default)]
    pub denies_per_match: Option<f64>,
    /// Kills per minute.
    #[serde(default)]
    pub kills_per_min: Option<f64>,
    /// Deaths per minute.
    #[serde(default)]
    pub deaths_per_min: Option<f64>,
    /// Assists per minute.
    #[serde(default)]
    pub assists_per_min: Option<f64>,
    /// Denies per minute.
    #[serde(default)]
    pub denies_per_min: Option<f64>,
    /// Souls per minute.
    #[serde(default)]
    pub networth_per_min: Option<f64>,
    /// Last hits per minute.
    #[serde(default)]
    pub last_hits_per_min: Option<f64>,
    /// Player damage per minute.
    #[serde(default)]
    pub damage_per_min: Option<f64>,
    /// Player damage per soul.
    #[serde(default)]
    pub damage_per_soul: Option<f64>,
    /// Damage mitigated per minute.
    #[serde(default)]
    pub damage_mitigated_per_min: Option<f64>,
    /// Damage taken per minute.
    #[serde(default)]
    pub damage_taken_per_min: Option<f64>,
    /// Damage taken per soul.
    #[serde(default)]
    pub damage_taken_per_soul: Option<f64>,
    /// Creeps per minute.
    #[serde(default)]
    pub creeps_per_min: Option<f64>,
    /// Objective damage per minute.
    #[serde(default)]
    pub obj_damage_per_min: Option<f64>,
    /// Objective damage per soul.
    #[serde(default)]
    pub obj_damage_per_soul: Option<f64>,
    /// Shot accuracy, `0`–`1`.
    #[serde(default)]
    pub accuracy: Option<f64>,
    /// Critical shot rate, `0`–`1`.
    #[serde(default)]
    pub crit_shot_rate: Option<f64>,
    /// Count of matches per MVP rank; shape not pinned.
    #[serde(default)]
    pub mvp_rank_counts: Vec<Value>,
    /// Matches with an MVP rating.
    #[serde(default)]
    pub mvp_rated_matches: Option<u32>,
    /// The match ids these statistics cover.
    #[serde(default)]
    pub matches: Vec<MatchId>,
    /// Permanent buffs taken.
    #[serde(default)]
    pub permanent_buffs: Option<u32>,
    /// Matches with a permanent buff.
    #[serde(default)]
    pub permanent_buff_matches: Option<u32>,
    /// Permanent buffs per minute.
    #[serde(default)]
    pub permanent_buffs_per_min: Option<f64>,
    /// Average seconds until the first permanent buff.
    #[serde(default)]
    pub avg_first_permanent_buff_time_s: Option<f64>,
}

/// One entry of `GET /v1/players/{id}/mate-stats`: results with one teammate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MateStats {
    /// The teammate's Steam account id.
    pub mate_id: AccountId,
    /// Matches won together.
    #[serde(default)]
    pub wins: Option<u32>,
    /// Matches played together.
    #[serde(default)]
    pub matches_played: Option<u32>,
    /// The match ids.
    #[serde(default)]
    pub matches: Vec<MatchId>,
}

/// One entry of `GET /v1/players/{id}/enemy-stats`: results against one opponent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnemyStats {
    /// The opponent's Steam account id.
    pub enemy_id: AccountId,
    /// Matches won against them.
    #[serde(default)]
    pub wins: Option<u32>,
    /// Matches played against them.
    #[serde(default)]
    pub matches_played: Option<u32>,
    /// The match ids.
    #[serde(default)]
    pub matches: Vec<MatchId>,
}
