//! Smoke tests against the live api.deadlock-api.com: every typed route once,
//! on an ingested match and a known account, so a change in the server's
//! schema shows up as a failed decode. They are ignored by default and do
//! nothing unless `DEADLOCK_API_LIVE=1`:
//!
//! ```sh
//! DEADLOCK_API_LIVE=1 cargo test --test live -- --ignored
//! ```
#![cfg(feature = "blocking")]

mod shape;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use deadlock_api::blocking::Client;
use deadlock_api::*;

const MATCH: MatchId = MatchId(122539002);
const ACCOUNT: AccountId = AccountId(167576350);
const ACCOUNT2: AccountId = AccountId(3226752420);

/// The client, or `None` when the live tests are not switched on.
fn api() -> Option<Client> {
    if std::env::var("DEADLOCK_API_LIVE").as_deref() != Ok("1") {
        eprintln!("skipped: set DEADLOCK_API_LIVE=1 to run against the live server");
        return None;
    }
    Some(
        Client::builder()
            .user_agent("deadlock-api-rs/live-tests")
            .build()
            .unwrap(),
    )
}

/// Runs one request at a time, a second after the previous one, like
/// `scripts/record-fixtures.sh`, so parallel test threads never burst.
fn paced<T>(request: impl FnOnce() -> T) -> T {
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(at) = *last {
        std::thread::sleep(Duration::from_secs(1).saturating_sub(at.elapsed()));
    }
    let out = request();
    *last = Some(Instant::now());
    out
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_metadata_decodes_with_no_shape_drift() {
    let Some(api) = api() else { return };
    let meta = paced(|| api.match_metadata(MATCH)).unwrap();
    assert_eq!(meta.match_info.match_id, MATCH.0);
    assert_eq!(meta.match_info.players.len(), 12);

    let path = format!("/v1/matches/{MATCH}/metadata");
    let raw: serde_json::Value =
        serde_json::from_slice(&paced(|| api.get_raw(&path, &[])).unwrap()).unwrap();
    let back = serde_json::to_value(&meta).unwrap();
    let mut diffs = Vec::new();
    shape::compare_shape("", &raw, &back, &mut diffs);
    assert!(diffs.is_empty(), "shape drift:\n{}", diffs.join("\n"));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_salts() {
    let Some(api) = api() else { return };
    assert_eq!(paced(|| api.match_salts(MATCH)).unwrap().match_id, MATCH.0);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_spectate_url_is_none_for_a_finished_match() {
    let Some(api) = api() else { return };
    assert_eq!(paced(|| api.match_spectate_url(MATCH)).unwrap(), None);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn active_matches() {
    let Some(api) = api() else { return };
    paced(|| api.active_matches(&ActiveMatchesQuery::default())).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn recently_fetched_matches() {
    let Some(api) = api() else { return };
    assert!(!paced(|| api.recently_fetched_matches()).unwrap().is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_history() {
    let Some(api) = api() else { return };
    assert!(!paced(|| api.match_history(ACCOUNT)).unwrap().is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn player_rank() {
    let Some(api) = api() else { return };
    paced(|| api.player_rank(ACCOUNT)).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn player_ranks() {
    let Some(api) = api() else { return };
    assert!(
        !paced(|| api.player_ranks(&[ACCOUNT, ACCOUNT2]))
            .unwrap()
            .is_empty()
    );
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn rank_distribution() {
    let Some(api) = api() else { return };
    assert!(
        !paced(|| api.rank_distribution(&MatchFilter::default()))
            .unwrap()
            .is_empty()
    );
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_stats() {
    let Some(api) = api() else { return };
    let stats = paced(|| api.hero_stats(&[ACCOUNT], &HeroStatsQuery::default())).unwrap();
    assert!(!stats.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn mate_stats() {
    let Some(api) = api() else { return };
    paced(|| api.mate_stats(ACCOUNT, &MateStatsQuery::default())).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn enemy_stats() {
    let Some(api) = api() else { return };
    paced(|| api.enemy_stats(ACCOUNT, &EnemyStatsQuery::default())).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn steam_profiles() {
    let Some(api) = api() else { return };
    assert!(
        !paced(|| api.steam_profiles(&[ACCOUNT, ACCOUNT2]))
            .unwrap()
            .is_empty()
    );
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn steam_search() {
    let Some(api) = api() else { return };
    let query = SteamSearchQuery {
        limit: Some(3),
        ..Default::default()
    };
    paced(|| api.steam_search("deadlock", &query)).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn leaderboard() {
    let Some(api) = api() else { return };
    assert!(
        !paced(|| api.leaderboard(Region::Europe))
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_leaderboard() {
    let Some(api) = api() else { return };
    let board = paced(|| api.hero_leaderboard(Region::Europe, HeroId(2))).unwrap();
    assert!(!board.entries.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn info() {
    let Some(api) = api() else { return };
    paced(|| api.info()).unwrap();
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn health() {
    let Some(api) = api() else { return };
    paced(|| api.health()).unwrap();
}
