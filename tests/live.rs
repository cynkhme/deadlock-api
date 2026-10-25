//! Smoke tests against the live api.deadlock-api.com: every typed route once,
//! on an ingested match, a match the server fetched recently and a known
//! account. Each response is fetched raw, decoded into its model and run
//! through the shape guard, so a field the server renames or retypes fails
//! the test instead of decoding to `None`. The typed methods' URLs are pinned
//! offline in `src/endpoints/mod.rs`. They are ignored by default and fail
//! unless `DEADLOCK_API_LIVE=1`:
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
use serde::Serialize;
use serde::de::DeserializeOwned;

const MATCH: MatchId = MatchId(122539002);
const ACCOUNT: AccountId = AccountId(167576350);
const ACCOUNT2: AccountId = AccountId(3226752420);

/// The client; panics unless the live tests are switched on, so a run with
/// `--ignored` but without the variable cannot pass on zero requests.
fn api() -> Client {
    assert_eq!(
        std::env::var("DEADLOCK_API_LIVE").as_deref(),
        Ok("1"),
        "set DEADLOCK_API_LIVE=1 to run the live tests"
    );
    Client::builder()
        .user_agent("deadlock-api-rs/live-tests")
        .build()
        .unwrap()
}

/// Runs one request at a time, a second after the previous one, like
/// the repository's `scripts/record-fixtures.sh`, so parallel test threads
/// never burst.
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

/// GETs `path` once, asserts the model carries every field of the response
/// and returns the decoded value.
fn checked<T: DeserializeOwned + Serialize>(api: &Client, path: &str, query: &[(&str, &str)]) -> T {
    let body = paced(|| api.get_raw(path, query)).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let diffs = shape::drift::<T>(&raw);
    assert!(
        diffs.is_empty(),
        "{path}: shape drift:\n{}",
        diffs.join("\n")
    );
    serde_json::from_value(raw).unwrap()
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_metadata() {
    let meta: MatchMetadata = checked(&api(), &format!("/v1/matches/{MATCH}/metadata"), &[]);
    assert_eq!(meta.match_info.match_id, MATCH);
    assert_eq!(meta.match_info.players.len(), 12);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn recently_fetched_matches_and_the_metadata_of_one() {
    let api = api();
    let recent: Vec<RecentMatch> = checked(&api, "/v1/matches/recently-fetched", &[]);
    let fresh = recent.first().expect("no recently fetched match").match_id;
    let meta: MatchMetadata = checked(&api, &format!("/v1/matches/{fresh}/metadata"), &[]);
    assert_eq!(meta.match_info.match_id, fresh);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_salts() {
    let salts: MatchSalts = checked(&api(), &format!("/v1/matches/{MATCH}/salts"), &[]);
    assert_eq!(salts.match_id, MATCH);
}

/// Typed, not raw: the `None` comes from the 400 the server sends for a
/// finished match, which only the typed method maps.
#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_spectate_url_is_none_for_a_finished_match() {
    let api = api();
    assert_eq!(paced(|| api.match_spectate_url(MATCH)).unwrap(), None);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn active_matches() {
    let _: Vec<ActiveMatch> = checked(&api(), "/v1/matches/active", &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn match_history() {
    let path = format!("/v1/players/{ACCOUNT}/match-history");
    let history: Vec<MatchHistoryEntry> = checked(&api(), &path, &[]);
    assert!(!history.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn player_rank() {
    let _: RankResponse = checked(&api(), &format!("/v1/players/{ACCOUNT}/rank"), &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn player_ranks() {
    let ids = format!("{ACCOUNT},{ACCOUNT2}");
    let ranks: Vec<AccountRank> = checked(&api(), "/v1/players/rank", &[("account_ids", &ids)]);
    assert!(!ranks.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn rank_distribution() {
    let path = "/v1/players/rank/distribution";
    let distribution: Vec<RankDistributionEntry> = checked(&api(), path, &[]);
    assert!(!distribution.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_stats() {
    let ids = ACCOUNT.to_string();
    let path = "/v1/players/hero-stats";
    let stats: Vec<HeroStats> = checked(&api(), path, &[("account_ids", &ids)]);
    assert!(!stats.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn mate_stats() {
    let _: Vec<MateStats> = checked(&api(), &format!("/v1/players/{ACCOUNT}/mate-stats"), &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn enemy_stats() {
    let _: Vec<EnemyStats> = checked(&api(), &format!("/v1/players/{ACCOUNT}/enemy-stats"), &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn steam_profiles() {
    let ids = format!("{ACCOUNT},{ACCOUNT2}");
    let profiles: Vec<SteamProfile> =
        checked(&api(), "/v1/players/steam", &[("account_ids", &ids)]);
    assert!(!profiles.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn steam_search() {
    let query = [("search_query", "deadlock"), ("limit", "3")];
    let _: Vec<SteamProfile> = checked(&api(), "/v1/players/steam-search", &query);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn leaderboard() {
    let board: Leaderboard = checked(&api(), "/v1/leaderboard/Europe", &[]);
    assert!(!board.entries.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_leaderboard() {
    let board: Leaderboard = checked(&api(), "/v1/leaderboard/Europe/2", &[]);
    assert!(!board.entries.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn info() {
    let _: ApiInfo = checked(&api(), "/v1/info", &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn health() {
    let _: Health = checked(&api(), "/v1/info/health", &[]);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn heroes() {
    let heroes: Vec<Hero> = checked(&api(), "/v1/assets/heroes", &[]);
    assert!(heroes.iter().any(|h| h.id == HeroId(1)));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero() {
    let hero: Hero = checked(&api(), "/v1/assets/heroes/1", &[]);
    assert_eq!(hero.id, HeroId(1));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_by_name_with_reserved_characters() {
    let hero: Hero = checked(&api(), "/v1/assets/heroes/by-name/Mo%20%26%20Krill", &[]);
    assert_eq!(hero.name, "Mo & Krill");
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn items() {
    let items: Vec<AssetItem> = checked(&api(), "/v1/assets/items", &[]);
    assert!(items.iter().all(|i| !matches!(i, AssetItem::Other(_))));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn item() {
    let item: AssetItem = checked(&api(), "/v1/assets/items/1548066885", &[]);
    assert_eq!(item.id(), Some(ItemId(1548066885)));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn item_by_class_name() {
    let item: AssetItem = checked(&api(), "/v1/assets/items/upgrade_clip_size", &[]);
    assert_eq!(item.class_name(), Some("upgrade_clip_size"));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn hero_items() {
    let items: Vec<AssetItem> = checked(&api(), "/v1/assets/items/by-hero-id/7", &[]);
    assert!(!items.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn items_by_slot() {
    let items: Vec<AssetItem> = checked(&api(), "/v1/assets/items/by-slot-type/spirit", &[]);
    assert!(!items.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn items_by_kind() {
    let items: Vec<AssetItem> = checked(&api(), "/v1/assets/items/by-type/weapon", &[]);
    assert!(items.iter().all(|i| matches!(i, AssetItem::Weapon(_))));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn ranks() {
    let ranks: Vec<Rank> = checked(&api(), "/v1/assets/ranks", &[]);
    assert!(ranks.iter().any(|r| r.tier == 0));
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn rank() {
    let rank: Rank = checked(&api(), "/v1/assets/ranks/1", &[]);
    assert_eq!(rank.tier, 1);
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn patches() {
    let patches: Vec<Patch> = checked(&api(), "/v1/patches", &[]);
    assert!(!patches.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn patch_feed() {
    let feed: Vec<FeedItem> = checked(&api(), "/v2/patches", &[]);
    assert!(!feed.is_empty());
}

#[test]
#[ignore = "live server; set DEADLOCK_API_LIVE=1"]
fn patch_big_days() {
    let days: Vec<String> = checked(&api(), "/v1/patches/big-days", &[]);
    assert!(!days.is_empty());
}
