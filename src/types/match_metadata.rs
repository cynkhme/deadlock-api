//! `GET /v1/matches/{id}/metadata`, typed from a real response rather than
//! the OpenAPI document, which leaves it untyped.
//!
//! The server does not document these fields; the types come from one
//! recorded match (122539002, 2026-10-24). To survive drift, only the fields
//! a match cannot lack are required; everything else is `Option` or a `Vec`
//! that defaults to empty, so an unexpected shape in one sub-object costs one
//! `None`, not the whole decode. Fields whose structure the sample does not
//! reveal are kept as [`serde_json::Value`].

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AccountId, HeroId};

/// The response: the match plus the hero selections around it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MatchMetadata {
    /// The match itself.
    pub match_info: MatchInfo,
    /// Hero builds picked by the players; an empty object in the sample, structure unknown.
    #[serde(default)]
    pub hero_build_ids: Option<Value>,
    /// Heroes selected before the match started; an empty object in the sample, structure unknown.
    #[serde(default)]
    pub pregame_hero_ids: Option<Value>,
    /// Heroes banned before the match started; empty in the sample.
    #[serde(default)]
    pub banned_hero_ids: Vec<u32>,
}

/// One match: outcome, players, objectives, movement and damage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MatchInfo {
    /// The match id.
    pub match_id: u64,
    /// Match length in seconds.
    pub duration_s: u32,
    /// The twelve players; fewer in custom or abandoned matches.
    pub players: Vec<Player>,
    /// Unix seconds when the match started.
    #[serde(default)]
    pub start_time: Option<u64>,
    /// Observed values in the 2026-10-24 sample: `0`. Undocumented upstream.
    #[serde(default)]
    pub match_outcome: Option<u32>,
    /// Observed values: `0` and `1`, the team index.
    #[serde(default)]
    pub winning_team: Option<u32>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub game_mode: Option<u32>,
    /// Observed values: `1`. Undocumented upstream.
    #[serde(default)]
    pub match_mode: Option<u32>,
    /// Version of the game mode the match ran on.
    #[serde(default)]
    pub game_mode_version: Option<u32>,
    /// Objectives destroyed during the match.
    #[serde(default)]
    pub objectives: Vec<Objective>,
    /// Mid-boss kills.
    #[serde(default)]
    pub mid_boss: Vec<MidBoss>,
    /// Pauses during the match.
    #[serde(default)]
    pub match_pauses: Vec<Pause>,
    /// Sampled player positions, see [`MatchPaths`].
    #[serde(default)]
    pub match_paths: Option<MatchPaths>,
    /// Sampled damage dealt, see [`DamageMatrix`].
    #[serde(default)]
    pub damage_matrix: Option<DamageMatrix>,
    /// Per-team entries.
    #[serde(default)]
    pub teams: Vec<Team>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub team_score: Vec<Value>,
    /// Bit mask of objectives team 0 destroyed.
    #[serde(default)]
    pub objectives_mask_team0: Option<u64>,
    /// Bit mask of objectives team 1 destroyed.
    #[serde(default)]
    pub objectives_mask_team1: Option<u64>,
    /// Average rank badge of team 0.
    #[serde(default)]
    pub average_badge_team0: Option<u32>,
    /// Average rank badge of team 1.
    #[serde(default)]
    pub average_badge_team1: Option<u32>,
    /// Whether the match was in the high-skill parties range.
    #[serde(default)]
    pub is_high_skill_range_parties: Option<bool>,
    /// Whether the match was in the low-priority pool.
    #[serde(default)]
    pub low_pri_pool: Option<bool>,
    /// Whether the match was in the new-player pool.
    #[serde(default)]
    pub new_player_pool: Option<bool>,
    /// Whether the match counted for rewards.
    #[serde(default)]
    pub rewards_eligible: Option<bool>,
    /// Whether the match was excluded from scoring.
    #[serde(default)]
    pub not_scored: Option<bool>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub ranked_type: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub rank_interval: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub bot_difficulty: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub corrupted_penalty_seed: Option<u64>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub legacy_objectives_mask: Option<Value>,
    /// Named custom statistics (`{"name", "id"}` entries); semantics unknown.
    #[serde(default)]
    pub custom_user_stats: Vec<Value>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub extra_messages: Vec<Value>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub match_tracked_stats: Vec<Value>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub street_brawl_rounds: Vec<Value>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub watched_death_replays: Vec<Value>,
}

/// One player's slot in the match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Player {
    /// The player's Steam account id.
    pub account_id: AccountId,
    /// The slot, `1`–`12`; referenced by [`Death::killer_player_slot`] and [`PlayerPath::player_slot`].
    pub player_slot: u32,
    /// Observed values: `0` and `1`, the team index.
    #[serde(default)]
    pub team: Option<u32>,
    /// The hero played.
    #[serde(default)]
    pub hero_id: Option<HeroId>,
    /// Kills at the end of the match.
    #[serde(default)]
    pub kills: Option<u32>,
    /// Deaths at the end of the match.
    #[serde(default)]
    pub deaths: Option<u32>,
    /// Assists at the end of the match.
    #[serde(default)]
    pub assists: Option<u32>,
    /// Net worth in souls at the end of the match.
    #[serde(default)]
    pub net_worth: Option<u64>,
    /// Last hits at the end of the match.
    #[serde(default)]
    pub last_hits: Option<u32>,
    /// Denies at the end of the match.
    #[serde(default)]
    pub denies: Option<u32>,
    /// Ability points earned.
    #[serde(default)]
    pub ability_points: Option<u32>,
    /// Observed values: `1`–`4`. Undocumented upstream.
    #[serde(default)]
    pub assigned_lane: Option<u32>,
    /// Level at the end of the match.
    #[serde(default)]
    pub level: Option<u32>,
    /// Observed values: `0` and `1`. Undocumented upstream.
    #[serde(default)]
    pub player_match_outcome: Option<u32>,
    /// Seconds into the match when the player abandoned it; `0` when they did not.
    #[serde(default)]
    pub abandon_match_time_s: Option<u32>,
    /// Whether this player was eligible for rewards.
    #[serde(default)]
    pub rewards_eligible: Option<bool>,
    /// Statistics sampled through the match.
    #[serde(default)]
    pub stats: Vec<PlayerStats>,
    /// Items bought, with the time of purchase.
    #[serde(default)]
    pub items: Vec<Item>,
    /// The player's deaths.
    #[serde(default)]
    pub death_details: Vec<Death>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub ability_stats: Vec<Value>,
    /// Pings (`{"ping_type", "ping_data", "game_time_s"}` entries); semantics unknown.
    #[serde(default)]
    pub pings: Vec<Value>,
    /// Hero-specific data (`hero_equips`, `hero_xp` in the sample); structure undocumented.
    #[serde(default)]
    pub hero_data: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub player_rank_data: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub mvp_rank: Option<Value>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub earned_holiday_award_2025: Option<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub stats_type_stat: Vec<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub book_rewards: Vec<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub player_tracked_stats: Vec<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub accolades: Vec<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub power_up_buffs: Vec<Value>,
    /// Shape unknown.
    #[serde(default)]
    pub hero_xp_rewards: Vec<Value>,
}

/// A sample of one player's statistics at [`PlayerStats::time_stamp_s`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PlayerStats {
    /// Seconds into the match this sample was taken.
    #[serde(default)]
    pub time_stamp_s: Option<u32>,
    /// Level at this time.
    #[serde(default)]
    pub level: Option<u32>,
    /// Net worth in souls.
    #[serde(default)]
    pub net_worth: Option<u64>,
    /// Kills so far.
    #[serde(default)]
    pub kills: Option<u32>,
    /// Deaths so far.
    #[serde(default)]
    pub deaths: Option<u32>,
    /// Assists so far.
    #[serde(default)]
    pub assists: Option<u32>,
    /// Denies so far.
    #[serde(default)]
    pub denies: Option<u32>,
    /// Lane creeps killed.
    #[serde(default)]
    pub creep_kills: Option<u32>,
    /// Neutral creeps killed.
    #[serde(default)]
    pub neutral_kills: Option<u32>,
    /// Lane creeps the player could have last-hit.
    #[serde(default)]
    pub possible_creeps: Option<u32>,
    /// Kills with bullets.
    #[serde(default)]
    pub bullet_kills: Option<u32>,
    /// Kills with melee.
    #[serde(default)]
    pub melee_kills: Option<u32>,
    /// Kills with abilities.
    #[serde(default)]
    pub ability_kills: Option<u32>,
    /// Kills with headshots.
    #[serde(default)]
    pub headshot_kills: Option<u32>,
    /// Ability points earned.
    #[serde(default)]
    pub ability_points: Option<u32>,
    /// Maximum health at this time.
    #[serde(default)]
    pub max_health: Option<u64>,
    /// Weapon power at this time.
    #[serde(default)]
    pub weapon_power: Option<u32>,
    /// Spirit (tech) power at this time.
    #[serde(default)]
    pub tech_power: Option<u32>,
    /// Shots that hit.
    #[serde(default)]
    pub shots_hit: Option<u32>,
    /// Shots that missed.
    #[serde(default)]
    pub shots_missed: Option<u32>,
    /// Bullets that hit a hero.
    #[serde(default)]
    pub hero_bullets_hit: Option<u32>,
    /// Bullets that hit a hero critically.
    #[serde(default)]
    pub hero_bullets_hit_crit: Option<u32>,
    /// Souls from players.
    #[serde(default)]
    pub gold_player: Option<u64>,
    /// Souls from player orbs.
    #[serde(default)]
    pub gold_player_orbs: Option<u64>,
    /// Souls from lane creeps.
    #[serde(default)]
    pub gold_lane_creep: Option<u64>,
    /// Souls from lane creep orbs.
    #[serde(default)]
    pub gold_lane_creep_orbs: Option<u64>,
    /// Souls from neutral creeps.
    #[serde(default)]
    pub gold_neutral_creep: Option<u64>,
    /// Souls from neutral creep orbs.
    #[serde(default)]
    pub gold_neutral_creep_orbs: Option<u64>,
    /// Souls from bosses.
    #[serde(default)]
    pub gold_boss: Option<u64>,
    /// Souls from boss orbs.
    #[serde(default)]
    pub gold_boss_orb: Option<u64>,
    /// Souls from treasure.
    #[serde(default)]
    pub gold_treasure: Option<u64>,
    /// Souls denied to the enemy.
    #[serde(default)]
    pub gold_denied: Option<u64>,
    /// Souls lost on death.
    #[serde(default)]
    pub gold_death_loss: Option<u64>,
    /// Damage dealt to lane creeps.
    #[serde(default)]
    pub creep_damage: Option<u64>,
    /// Damage dealt to players.
    #[serde(default)]
    pub player_damage: Option<u64>,
    /// Damage dealt to neutral creeps.
    #[serde(default)]
    pub neutral_damage: Option<u64>,
    /// Damage dealt to bosses.
    #[serde(default)]
    pub boss_damage: Option<u64>,
    /// Damage taken from players.
    #[serde(default)]
    pub player_damage_taken: Option<u64>,
    /// Damage dealt to self.
    #[serde(default)]
    pub self_damage: Option<u64>,
    /// Damage mitigated.
    #[serde(default)]
    pub damage_mitigated: Option<u64>,
    /// Damage absorbed by barriers.
    #[serde(default)]
    pub damage_absorbed: Option<u64>,
    /// Barrier absorption provided to others.
    #[serde(default)]
    pub absorption_provided: Option<u64>,
    /// Healing done to players.
    #[serde(default)]
    pub player_healing: Option<u64>,
    /// Healing done to self.
    #[serde(default)]
    pub self_healing: Option<u64>,
    /// Healing done to teammates.
    #[serde(default)]
    pub teammate_healing: Option<u64>,
    /// Barriers given to self.
    #[serde(default)]
    pub player_barriering: Option<u64>,
    /// Barriers given to teammates.
    #[serde(default)]
    pub teammate_barriering: Option<u64>,
    /// Enemy healing prevented.
    #[serde(default)]
    pub heal_prevented: Option<u64>,
    /// Own healing lost to anti-heal.
    #[serde(default)]
    pub heal_lost: Option<u64>,
    /// Souls by source; shape unknown.
    #[serde(default)]
    pub gold_sources: Vec<Value>,
    /// Custom statistics at this time; shape unknown.
    #[serde(default)]
    pub custom_user_stats: Vec<Value>,
}

/// An item purchase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Item {
    /// Seconds into the match the item was bought.
    #[serde(default)]
    pub game_time_s: Option<u32>,
    /// The item id.
    #[serde(default)]
    pub item_id: Option<u64>,
    /// The upgrade id, `0` when none.
    #[serde(default)]
    pub upgrade_id: Option<u64>,
    /// Seconds into the match the item was sold, `0` when kept.
    #[serde(default)]
    pub sold_time_s: Option<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub flags: Option<u32>,
    /// The ability the item was imbued into, `0` when none.
    #[serde(default)]
    pub imbued_ability_id: Option<u64>,
    /// Observed values: `0` and `65536`. Undocumented upstream.
    #[serde(default)]
    pub upgrade_info: Option<u64>,
}

/// One death of a player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Death {
    /// Seconds into the match.
    #[serde(default)]
    pub game_time_s: Option<u32>,
    /// Seconds the killing took.
    #[serde(default)]
    pub time_to_kill_s: Option<f64>,
    /// The killer's [`Player::player_slot`]; absent when there was no killer.
    #[serde(default)]
    pub killer_player_slot: Option<u32>,
    /// Where the player died.
    #[serde(default)]
    pub death_pos: Option<Pos>,
    /// Where the killer stood.
    #[serde(default)]
    pub killer_pos: Option<Pos>,
    /// Seconds until respawn.
    #[serde(default)]
    pub death_duration_s: Option<u32>,
}

/// A position in world units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pos {
    /// East-west.
    pub x: f64,
    /// North-south.
    pub y: f64,
    /// Height; `0` where the server sends none.
    #[serde(default)]
    pub z: f64,
}

impl Pos {
    /// Horizontal distance to `other`, ignoring height.
    pub fn distance(&self, other: &Pos) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// A destroyed objective.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Objective {
    /// Which objective, as the server numbers them.
    #[serde(default)]
    pub team_objective_id: Option<u32>,
    /// The team that owned it.
    #[serde(default)]
    pub team: Option<u32>,
    /// Seconds into the match it fell.
    #[serde(default)]
    pub destroyed_time_s: Option<u32>,
    /// Seconds into the match it first took damage.
    #[serde(default)]
    pub first_damage_time_s: Option<u32>,
    /// Damage from creeps.
    #[serde(default)]
    pub creep_damage: Option<u64>,
    /// Creep damage mitigated; `null` in the sample.
    #[serde(default)]
    pub creep_damage_mitigated: Option<u64>,
    /// Damage from players.
    #[serde(default)]
    pub player_damage: Option<u64>,
    /// Player damage mitigated; `null` in the sample.
    #[serde(default)]
    pub player_damage_mitigated: Option<u64>,
    /// Spirit damage from players.
    #[serde(default)]
    pub player_spirit_damage: Option<u64>,
    /// `null` in the sample; structure unknown.
    #[serde(default)]
    pub legacy_objective_id: Option<Value>,
}

/// A mid-boss kill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MidBoss {
    /// The team that killed it.
    #[serde(default)]
    pub team_killed: Option<u32>,
    /// The team that claimed the reward.
    #[serde(default)]
    pub team_claimed: Option<u32>,
    /// Seconds into the match.
    #[serde(default)]
    pub destroyed_time_s: Option<u32>,
}

/// A pause; empty in the sample, shape from the previous client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Pause {
    /// Seconds into the match the pause began.
    #[serde(default)]
    pub game_time_s: Option<u32>,
    /// How long it lasted.
    #[serde(default)]
    pub pause_duration_s: Option<u32>,
}

/// Sampled positions of every player, quantised to a grid.
///
/// Positions are stored as grid cells; [`MatchPaths::position`] maps a cell
/// back to world units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MatchPaths {
    /// Format version.
    #[serde(default)]
    pub version: Option<u32>,
    /// Seconds between samples; the server sends it as a float (`1.0`).
    #[serde(default)]
    pub interval_s: Option<f64>,
    /// Grid cells per world unit range along x.
    pub x_resolution: f64,
    /// Grid cells per world unit range along y.
    pub y_resolution: f64,
    /// One path per player.
    #[serde(default)]
    pub paths: Vec<PlayerPath>,
}

impl MatchPaths {
    /// The path of the player in `player_slot`.
    pub fn player(&self, player_slot: u32) -> Option<&PlayerPath> {
        self.paths.iter().find(|p| p.player_slot == player_slot)
    }

    /// The world position of sample `i` on `path`.
    ///
    /// # Panics
    ///
    /// When `i` is out of range of `path.x_pos` or `path.y_pos`.
    pub fn position(&self, path: &PlayerPath, i: usize) -> Pos {
        Pos {
            x: path.x_min
                + f64::from(path.x_pos[i]) * (path.x_max - path.x_min) / self.x_resolution,
            y: path.y_min
                + f64::from(path.y_pos[i]) * (path.y_max - path.y_min) / self.y_resolution,
            z: 0.0,
        }
    }
}

/// One player's sampled path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PlayerPath {
    /// The player's [`Player::player_slot`].
    pub player_slot: u32,
    /// World x of grid cell 0.
    pub x_min: f64,
    /// World y of grid cell 0.
    pub y_min: f64,
    /// World x of the last grid cell.
    pub x_max: f64,
    /// World y of the last grid cell.
    pub y_max: f64,
    /// Grid x per sample.
    #[serde(default)]
    pub x_pos: Vec<u32>,
    /// Grid y per sample.
    #[serde(default)]
    pub y_pos: Vec<u32>,
    /// Health per sample.
    #[serde(default)]
    pub health: Vec<u32>,
    /// Observed values: `0`. Undocumented upstream.
    #[serde(default)]
    pub combat_type: Vec<u32>,
    /// Observed values: `6`. Undocumented upstream.
    #[serde(default)]
    pub move_type: Vec<u32>,
}

/// Damage dealt by each player, sampled over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DamageMatrix {
    /// Seconds into the match of each sample.
    #[serde(default)]
    pub sample_time_s: Vec<u32>,
    /// One entry per dealing player.
    #[serde(default)]
    pub damage_dealers: Vec<DamageDealer>,
    /// Names of the damage sources (`source_name`, `stat_type`); shape unknown.
    #[serde(default)]
    pub source_details: Option<Value>,
}

/// Damage dealt by one player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DamageDealer {
    /// The dealer's [`Player::player_slot`].
    #[serde(default)]
    pub dealer_player_slot: Option<u32>,
    /// Per-source damage (`damage_to_players`, `source_details_index`); shape unknown.
    #[serde(default)]
    pub damage_sources: Vec<Value>,
}

/// A team entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Team {
    /// The team index, `0` or `1`.
    #[serde(default)]
    pub team: Option<u32>,
    /// Empty in the sample; shape unknown.
    #[serde(default)]
    pub team_tracked_stats: Vec<Value>,
}
