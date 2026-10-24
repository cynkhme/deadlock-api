//! Every fixture decodes into its model, and the metadata model tolerates
//! what the server may drop or add.

use deadlock_api::{AccountId, MatchMetadata};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn match_metadata_decodes() {
    let m: MatchMetadata = serde_json::from_slice(&fixture("match_metadata.json")).unwrap();
    assert_eq!(m.match_info.match_id, 122539002);
    assert_eq!(m.match_info.duration_s, 1616);
    assert_eq!(m.match_info.players.len(), 2);
    let p = &m.match_info.players[0];
    assert_eq!(p.account_id, AccountId(123324885));
    assert_eq!(p.player_slot, 5);
    assert_eq!(p.death_details[0].killer_player_slot, Some(11));
    assert_eq!(p.death_details[0].game_time_s, Some(495));
    assert_eq!(p.items[0].item_id, Some(267712790));
    assert_eq!(p.stats[0].time_stamp_s, Some(180));
    assert_eq!(m.match_info.winning_team, Some(0));
    assert_eq!(m.match_info.start_time, Some(1792705407));
    assert_eq!(m.match_info.objectives[0].team_objective_id, Some(3));
    assert_eq!(m.match_info.mid_boss[0].team_killed, Some(0));
}

/// Drift guard for the tolerant typing: every optional field the
/// model carries is present in the recorded sample, so a `None` in a live
/// response means the server changed, not that the field never existed.
#[test]
fn tolerant_fields_are_all_present_in_the_sample() {
    let m: MatchMetadata = serde_json::from_slice(&fixture("match_metadata.json")).unwrap();
    let i = &m.match_info;
    for (name, present) in [
        ("start_time", i.start_time.is_some()),
        ("winning_team", i.winning_team.is_some()),
        ("match_outcome", i.match_outcome.is_some()),
        ("game_mode", i.game_mode.is_some()),
        ("match_mode", i.match_mode.is_some()),
        ("objectives", !i.objectives.is_empty()),
        ("mid_boss", !i.mid_boss.is_empty()),
        ("match_paths", i.match_paths.is_some()),
        ("damage_matrix", i.damage_matrix.is_some()),
        ("teams", !i.teams.is_empty()),
    ] {
        assert!(present, "match_info.{name}");
    }
    let p = &i.players[0];
    for (name, present) in [
        ("team", p.team.is_some()),
        ("hero_id", p.hero_id.is_some()),
        ("kills", p.kills.is_some()),
        ("deaths", p.deaths.is_some()),
        ("assists", p.assists.is_some()),
        ("net_worth", p.net_worth.is_some()),
        ("level", p.level.is_some()),
        ("stats", !p.stats.is_empty()),
        ("items", !p.items.is_empty()),
        ("death_details", !p.death_details.is_empty()),
        ("hero_data", p.hero_data.is_some()),
    ] {
        assert!(present, "players[0].{name}");
    }
    assert!(p.stats[0].time_stamp_s.is_some() && p.stats[0].net_worth.is_some());
    assert!(p.items[0].item_id.is_some() && p.items[0].game_time_s.is_some());
    assert!(p.death_details[0].game_time_s.is_some() && p.death_details[0].death_pos.is_some());
    let paths = i.match_paths.as_ref().unwrap();
    assert!(!paths.paths.is_empty() && !paths.paths[0].x_pos.is_empty());
    let dm = i.damage_matrix.as_ref().unwrap();
    assert!(!dm.sample_time_s.is_empty() && !dm.damage_dealers.is_empty());
}

#[test]
fn unknown_top_level_field_is_ignored_and_missing_nested_is_none() {
    let json = r#"{"match_info":{"match_id":1,"duration_s":2,"players":[{"account_id":3,"player_slot":4}],"brand_new":true}}"#;
    let m: MatchMetadata = serde_json::from_str(json).unwrap();
    assert!(m.match_info.players[0].items.is_empty());
    assert!(m.match_info.winning_team.is_none());
    assert!(m.hero_build_ids.is_none());
    assert!(m.banned_hero_ids.is_empty());
}

#[test]
fn match_paths_helpers_match_the_old_formulas() {
    let json = r#"{"x_resolution":10.0,"y_resolution":10.0,"paths":[{"player_slot":1,"x_min":0.0,"y_min":0.0,"x_max":100.0,"y_max":50.0,"x_pos":[5],"y_pos":[10],"health":[100]}]}"#;
    let paths: deadlock_api::MatchPaths = serde_json::from_str(json).unwrap();
    let path = paths.player(1).unwrap();
    let pos = paths.position(path, 0);
    assert_eq!((pos.x, pos.y), (50.0, 50.0));
    assert!(paths.player(2).is_none());
    let a = deadlock_api::Pos {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let b = deadlock_api::Pos {
        x: 3.0,
        y: 4.0,
        z: 9.0,
    };
    assert_eq!(a.distance(&b), 5.0);
}

mod other_endpoints {
    use super::fixture;
    use deadlock_api::{
        AccountId, AccountRank, ActiveMatch, ApiInfo, EnemyStats, Health, HeroId, HeroStats,
        Leaderboard, MatchHistoryEntry, MatchSalts, MateStats, RankDistributionEntry, RankResponse,
        RecentMatch, Region, SteamProfile,
    };

    fn decode<T: serde::de::DeserializeOwned>(name: &str) -> T {
        serde_json::from_slice(&fixture(name)).unwrap()
    }

    #[test]
    fn match_salts_decodes() {
        let s: MatchSalts = decode("match_salts.json");
        assert_eq!(s.match_id, 122539002);
        assert_eq!(s.cluster_id, Some(429));
        assert_eq!(s.metadata_salt, Some(1910618982));
        assert_eq!(s.replay_salt, Some(1130965774));
        assert!(s.metadata_url.as_deref().unwrap().ends_with(".meta.bz2"));
    }

    #[test]
    fn active_matches_decode() {
        let v: Vec<ActiveMatch> = decode("active_matches.json");
        assert_eq!(v.len(), 3);
        let m = &v[0];
        assert_eq!(m.match_mode, Some(1));
        assert_eq!(m.match_mode_parsed.as_deref(), Some("Unranked"));
        assert_eq!(m.region_mode_parsed.as_deref(), Some("row"));
        assert_eq!(m.winning_team, None);
        assert_eq!(m.duration_s, None);
        assert!(m.lobby_id.unwrap() > u64::from(u32::MAX));
        assert_eq!(m.players[0].team_parsed.as_deref(), Some("Team0"));
        assert_eq!(m.team_stats.len(), 2);
    }

    #[test]
    fn recently_fetched_decode() {
        let v: Vec<RecentMatch> = decode("recently_fetched.json");
        assert_eq!(v.len(), 3);
        assert!(v[0].match_id > 0);
        assert_eq!(v[0].average_badge, None);
        assert!(!v[0].players.is_empty());
    }

    #[test]
    fn match_history_decodes() {
        let v: Vec<MatchHistoryEntry> = decode("match_history.json");
        assert_eq!(v[0].account_id, AccountId(167576350));
        assert_eq!(v[0].match_id, 123971228);
        assert_eq!(v[0].hero_id, Some(HeroId(80)));
        assert_eq!(v[0].player_kills, Some(7));
        assert_eq!(v[0].brawl_score_team0, None);
        assert_eq!(v[0].low_pri_pool, Some(false));
    }

    #[test]
    fn ranks_decode() {
        let r: RankResponse = decode("player_rank.json");
        assert_eq!((r.badge, r.rank, r.subrank), (Some(91), Some(9), Some(1)));
        let v: Vec<AccountRank> = decode("player_ranks.json");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].account_id, AccountId(167576350));
        let d: Vec<RankDistributionEntry> = decode("rank_distribution.json");
        assert_eq!(d[0].badge, Some(11));
        assert_eq!(d[0].players, Some(2006));
    }

    #[test]
    fn hero_mate_enemy_stats_decode() {
        let h: Vec<HeroStats> = decode("hero_stats.json");
        assert_eq!(h[0].hero_id, HeroId(1));
        assert_eq!(h[0].matches_played, Some(380));
        assert!(h[0].ending_level.unwrap() > 30.0);
        assert_eq!(h[0].matches.len(), 380);
        let m: Vec<MateStats> = decode("mate_stats.json");
        assert_eq!(m[0].mate_id, AccountId(1180548075));
        assert_eq!((m[0].wins, m[0].matches_played), (Some(67), Some(126)));
        let e: Vec<EnemyStats> = decode("enemy_stats.json");
        assert_eq!(e[0].enemy_id, AccountId(4028775));
    }

    #[test]
    fn steam_profiles_decode() {
        let v: Vec<SteamProfile> = decode("steam_profiles.json");
        assert_eq!(v[0].account_id, AccountId(167576350));
        assert_eq!(v[0].personaname.as_deref(), Some("player-1"));
        assert_eq!(v[0].last_updated.as_deref(), Some("2026-10-19T22:01:03Z"));
        assert_eq!(v[0].friends[0].account_id, AccountId(4000001000));
        let s: Vec<SteamProfile> = decode("steam_search.json");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].last_team_avg_badge, None);
    }

    #[test]
    fn leaderboards_decode() {
        let l: Leaderboard = decode("leaderboard.json");
        assert_eq!(l.entries.len(), 3);
        assert_eq!(l.entries[0].rank, Some(1));
        assert_eq!(l.entries[0].top_hero_ids, vec![HeroId(13)]);
        let h: Leaderboard = decode("hero_leaderboard.json");
        assert_eq!(
            h.entries[0].possible_account_ids,
            vec![AccountId(1277112179)]
        );
    }

    #[test]
    fn info_and_health_decode() {
        let i: ApiInfo = decode("info.json");
        assert_eq!(i.fetched_matches_per_day, Some(487713));
        assert!(i.table_sizes["player_match_roster"].rows.unwrap() > 1_000_000);
        let h: Health = decode("health.json");
        assert!(h.services["clickhouse"]);
    }

    #[test]
    fn region_round_trips_and_keeps_unknown_values() {
        assert_eq!(
            serde_json::from_str::<Region>(r#""Europe""#).unwrap(),
            Region::Europe
        );
        assert_eq!(
            serde_json::to_string(&Region::NAmerica).unwrap(),
            r#""NAmerica""#
        );
        let unknown: Region = serde_json::from_str(r#""Mars""#).unwrap();
        assert_eq!(unknown, Region::Unknown("Mars".into()));
        assert_eq!(serde_json::to_string(&unknown).unwrap(), r#""Mars""#);
        assert_eq!(Region::SAmerica.as_path_segment(), "SAmerica");
        assert_eq!(unknown.as_path_segment(), "Mars");
    }
}
