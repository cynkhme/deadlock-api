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
