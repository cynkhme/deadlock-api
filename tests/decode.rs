//! Every fixture decodes into its model, and the metadata model tolerates
//! what the server may drop or add.

mod shape;

use deadlock_api::{AbilityStat, AccountId, MatchId, MatchMetadata};
use shape::drift;

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
    assert_eq!(m.match_info.match_id, MatchId(122539002));
    assert_eq!(m.match_info.duration_s, 1616);
    assert_eq!(m.match_info.players.len(), 2);
    let p = &m.match_info.players[0];
    assert_eq!(p.account_id, AccountId(441058371));
    assert_eq!(p.player_slot, 10);
    assert_eq!(p.kills, Some(5));
    assert_eq!(p.mvp_rank, Some(3));
    assert_eq!(m.match_info.players[1].mvp_rank, Some(1));
    let ability: &AbilityStat = &p.ability_stats[0];
    assert_eq!(ability.ability_id, Some(2521902222));
    assert_eq!(p.ability_stats[0].ability_value, Some(91));
    assert_eq!(m.match_info.players[1].ability_stats.len(), 1);
    assert_eq!(p.death_details[0].killer_player_slot, Some(4));
    assert_eq!(p.death_details[0].game_time_s, Some(302));
    assert_eq!(p.items[0].item_id, Some(2521902222));
    assert_eq!(p.stats[0].time_stamp_s, Some(180));
    assert_eq!(m.match_info.winning_team, Some(0));
    assert_eq!(m.match_info.start_time, Some(1792705407));
    assert_eq!(m.match_info.objectives[0].team_objective_id, Some(3));
    assert_eq!(m.match_info.mid_boss[0].team_killed, Some(0));
}

/// Drift guard for the tolerant typing, over every recorded
/// response: each fixture, decoded and serialized back, must carry every key
/// of the raw fixture with a value where the raw has one. A `None` or empty
/// list where the sample has data means a field was renamed or mistyped
/// upstream, or mistyped here. A fixture added without a model fails too.
#[test]
fn every_fixture_round_trips_without_shape_drift() {
    use deadlock_api::*;
    type Check = fn(&serde_json::Value) -> Vec<String>;
    let checks: [(&str, Check); 17] = [
        ("match_metadata.json", drift::<MatchMetadata>),
        ("match_salts.json", drift::<MatchSalts>),
        ("active_matches.json", drift::<Vec<ActiveMatch>>),
        ("recently_fetched.json", drift::<Vec<RecentMatch>>),
        ("match_history.json", drift::<Vec<MatchHistoryEntry>>),
        ("player_rank.json", drift::<RankResponse>),
        ("player_ranks.json", drift::<Vec<AccountRank>>),
        (
            "rank_distribution.json",
            drift::<Vec<RankDistributionEntry>>,
        ),
        ("hero_stats.json", drift::<Vec<HeroStats>>),
        ("mate_stats.json", drift::<Vec<MateStats>>),
        ("enemy_stats.json", drift::<Vec<EnemyStats>>),
        ("steam_profiles.json", drift::<Vec<SteamProfile>>),
        ("steam_search.json", drift::<Vec<SteamProfile>>),
        ("leaderboard.json", drift::<Leaderboard>),
        ("hero_leaderboard.json", drift::<Leaderboard>),
        ("info.json", drift::<ApiInfo>),
        ("health.json", drift::<Health>),
    ];
    // An error body, not a model.
    let not_models = ["spectate_url_400.json"];

    let dir = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
    let unchecked: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".json"))
        .filter(|name| !not_models.contains(&name.as_str()))
        .filter(|name| !checks.iter().any(|(checked, _)| checked == name))
        .collect();
    assert!(
        unchecked.is_empty(),
        "fixtures without a check: {unchecked:?}"
    );

    let mut diffs = Vec::new();
    for (name, check) in checks {
        let raw: serde_json::Value = serde_json::from_slice(&fixture(name)).unwrap();
        diffs.extend(check(&raw).into_iter().map(|d| format!("{name}{d}")));
    }
    assert!(diffs.is_empty(), "shape drift:\n{}", diffs.join("\n"));
}

#[test]
fn drift_reports_carry_no_response_values() {
    use deadlock_api::SteamProfile;
    let mut diffs = Vec::new();
    shape::compare_shape(
        "",
        &serde_json::json!({"name": "secret-name", "id": 4000000001u64, "flag": "secret-flag"}),
        &serde_json::json!({"name": "other-name", "id": 4000000002u64, "flag": true}),
        &mut diffs,
    );
    let retyped = serde_json::json!({"account_id": 7, "personaname": 4000000003u64});
    diffs.extend(drift::<SteamProfile>(&retyped));
    let escaped = serde_json::json!({"account_id": "a \"secret\" tail"});
    diffs.extend(drift::<SteamProfile>(&escaped));
    diffs.extend(drift::<SteamProfile>(&serde_json::json!({})));
    let report = diffs.join("\n");
    for value in ["secret", "other", "4000000001", "4000000002", "4000000003"] {
        assert!(!report.contains(value), "{value} in:\n{report}");
    }
    assert!(report.contains("/name: "), "{report}");
    assert!(report.contains("/flag: "), "{report}");
    assert!(report.contains("`account_id`"), "{report}");
}

#[test]
fn wrong_typed_sub_object_degrades_to_none_not_a_decode_error() {
    let json = r#"{"match_info":{"match_id":1,"duration_s":2,"players":[{"account_id":3,"player_slot":4,
        "stats":[{"time_stamp_s":"nope"},{"time_stamp_s":180}],"items":"not a list","death_details":[{"game_time_s":[]}]}],
        "match_paths":{"paths":[]},"damage_matrix":7,"objectives":[{"team":"red"}]}}"#;
    let m: MatchMetadata = serde_json::from_str(json).unwrap();
    let p = &m.match_info.players[0];
    assert_eq!(
        p.stats.len(),
        1,
        "the wrong-typed sample is skipped, the good one kept"
    );
    assert!(p.items.is_empty());
    assert!(p.death_details.is_empty());
    assert!(
        m.match_info.match_paths.is_none(),
        "match_paths without resolutions is None, not an error"
    );
    assert!(m.match_info.damage_matrix.is_none());
    assert!(m.match_info.objectives.is_empty());
}

#[test]
fn wrong_typed_scalar_field_is_still_a_decode_error() {
    let json = r#"{"match_info":{"match_id":1,"duration_s":2,"players":[{"account_id":3,"player_slot":4,"kills":"7"}]}}"#;
    assert!(serde_json::from_str::<MatchMetadata>(json).is_err());
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
        Leaderboard, MatchHistoryEntry, MatchId, MatchSalts, MateStats, RankDistributionEntry,
        RankResponse, RecentMatch, Region, SteamProfile,
    };

    fn decode<T: serde::de::DeserializeOwned>(name: &str) -> T {
        serde_json::from_slice(&fixture(name)).unwrap()
    }

    #[test]
    fn match_salts_decodes() {
        let s: MatchSalts = decode("match_salts.json");
        assert_eq!(s.match_id, MatchId(122539002));
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
        assert!(v[0].match_id > MatchId(0));
        assert_eq!(v[0].average_badge, None);
        assert!(!v[0].players.is_empty());
    }

    #[test]
    fn match_history_decodes() {
        let v: Vec<MatchHistoryEntry> = decode("match_history.json");
        assert_eq!(v[0].account_id, AccountId(167576350));
        assert_eq!(v[0].match_id, MatchId(123971228));
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
        assert!(h[0].matches.iter().all(|m: &MatchId| m.0 > 0));
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

/// Every list field of the metadata, set to `null`, decodes as if it were
/// `[]`; only `players`, which a match cannot lack, stays required.
#[test]
fn null_list_decodes_as_an_empty_list() {
    fn list_pointers(v: &serde_json::Value, at: &str, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(map) => {
                for (k, child) in map {
                    let here = format!("{at}/{k}");
                    if child.is_array() {
                        out.push(here.clone());
                    }
                    list_pointers(child, &here, out);
                }
            }
            serde_json::Value::Array(items) => {
                for (i, child) in items.iter().enumerate() {
                    list_pointers(child, &format!("{at}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    let raw: serde_json::Value = serde_json::from_slice(&fixture("match_metadata.json")).unwrap();
    let mut pointers = Vec::new();
    list_pointers(&raw, "", &mut pointers);
    pointers.retain(|p| p != "/match_info/players");
    assert!(pointers.len() > 20, "{} list fields found", pointers.len());

    let decode_with = |pointer: &str, value: serde_json::Value| {
        let mut v = raw.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        serde_json::from_value::<MatchMetadata>(v).map(|m| serde_json::to_value(&m).unwrap())
    };
    let mut failures = Vec::new();
    for pointer in &pointers {
        let empty = decode_with(pointer, serde_json::json!([])).unwrap();
        match decode_with(pointer, serde_json::Value::Null) {
            Ok(null) if null == empty => {}
            // Inside a field kept as `Value` the null is carried through as is.
            Ok(null) if null.pointer(pointer) == Some(&serde_json::Value::Null) => {}
            Ok(_) => failures.push(format!("{pointer}: null decodes differently from []")),
            Err(e) => failures.push(format!("{pointer}: {e}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
