//! One struct per route, each implementing [`Endpoint`](crate::endpoint::Endpoint),
//! and the macro that gives both clients the same convenience methods.

pub(crate) mod assets;
pub(crate) mod info;
pub(crate) mod leaderboard;
pub(crate) mod matches;
pub(crate) mod patches;
pub(crate) mod players;
pub(crate) mod steam;

/// The `# Errors` section shared by every convenience method.
macro_rules! errors_doc {
    () => {
        "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the request; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON; [`Error::InvalidRequest`](crate::Error::InvalidRequest) when an argument does not form a URL."
    };
}
pub(crate) use errors_doc;

/// Emits the typed convenience methods on a client.
///
/// `endpoint_methods!(sync)` emits `fn name(&self, ..) -> Result<T>` calling
/// `self.send(&endpoint)`; `endpoint_methods!(async)` emits `async fn` with
/// `.await`. The route list lives here once, so the two clients cannot
/// drift. Three kinds of entry: `plain`; `list`, whose id slice short-circuits
/// to an empty `Vec` without a request when empty; and `optional`, whose
/// 400 answer is `Ok(None)` (the server says "cannot be live" for a finished
/// match, which is an outcome, not an error).
macro_rules! endpoint_methods {
    ($mode:ident) => {
        $crate::endpoints::endpoint_methods! { @$mode
            plain match_metadata(id: MatchId) -> MatchMetadata = $crate::endpoints::matches::GetMatchMetadata { id };
                "Match metadata: players, objectives, paths and damage, typed from a real response. The one cached route."
                "MatchId(122539002)"
            plain match_salts(id: MatchId) -> MatchSalts = $crate::endpoints::matches::GetMatchSalts { id };
                "Where the match's replay and metadata files live; `replay_salt` is `None` until the replay exists."
                "MatchId(122539002)"
            optional match_spectate_url(id: MatchId) -> SpectateUrl = $crate::endpoints::matches::GetSpectateUrl { id };
                "The broadcast URL of a live match; `Ok(None)` when the match is not live."
                "MatchId(122539002)"
            plain active_matches(query: &ActiveMatchesQuery) -> Vec<ActiveMatch> = $crate::endpoints::matches::ListActiveMatches { query: query.clone() };
                "Matches in progress, optionally filtered by player."
                "&ActiveMatchesQuery::default()"
            plain recently_fetched_matches() -> Vec<RecentMatch> = $crate::endpoints::matches::ListRecentlyFetched;
                "Matches the server fetched most recently."
                ""
            plain match_history(account: AccountId) -> Vec<MatchHistoryEntry> = $crate::endpoints::players::GetMatchHistory { account };
                "A player's recent matches, newest first."
                "AccountId(167576350)"
            plain player_rank(account: AccountId) -> RankResponse = $crate::endpoints::players::GetPlayerRank { account };
                "A player's current rank badge."
                "AccountId(167576350)"
            list player_ranks(ids: &[AccountId]) -> Vec<AccountRank> = $crate::endpoints::players::GetPlayerRanks { ids: ids.to_vec() };
                "Rank badges of several players in one request."
                "&[AccountId(167576350)]"
            plain rank_distribution(filter: &MatchFilter) -> Vec<RankDistributionEntry> = $crate::endpoints::players::GetRankDistribution { filter: filter.clone() };
                "How many players hold each badge, over the matches the filter selects."
                "&MatchFilter::default()"
            list hero_stats(ids: &[AccountId], query: &HeroStatsQuery) -> Vec<HeroStats> = $crate::endpoints::players::GetHeroStats { ids: ids.to_vec(), query: query.clone() };
                "Per-hero statistics of several players over the matches the query selects."
                "&[AccountId(167576350)], &HeroStatsQuery::default()"
            plain mate_stats(account: AccountId, query: &MateStatsQuery) -> Vec<MateStats> = $crate::endpoints::players::GetMateStats { account, query: query.clone() };
                "A player's results with each teammate over the matches the query selects."
                "AccountId(167576350), &MateStatsQuery::default()"
            plain enemy_stats(account: AccountId, query: &EnemyStatsQuery) -> Vec<EnemyStats> = $crate::endpoints::players::GetEnemyStats { account, query: query.clone() };
                "A player's results against each opponent over the matches the query selects."
                "AccountId(167576350), &EnemyStatsQuery::default()"
            list steam_profiles(ids: &[AccountId]) -> Vec<SteamProfile> = $crate::endpoints::steam::GetSteamProfiles { ids: ids.to_vec() };
                "Steam profiles (names, avatars, friends) of several players."
                "&[AccountId(167576350)]"
            plain steam_search(search_query: &str, query: &SteamSearchQuery) -> Vec<SteamProfile> = $crate::endpoints::steam::SearchSteam { search_query: search_query.to_owned(), query: query.clone() };
                "Steam profiles whose name matches `search_query`; an empty query is sent as is and the server decides."
                "\"deadlock\", &SteamSearchQuery::default()"
            plain leaderboard(region: Region) -> Leaderboard = $crate::endpoints::leaderboard::GetLeaderboard { region };
                "The ranked leaderboard of a region."
                "Region::Europe"
            plain hero_leaderboard(region: Region, hero: HeroId) -> Leaderboard = $crate::endpoints::leaderboard::GetHeroLeaderboard { region, hero };
                "The leaderboard of a region for one hero."
                "Region::Europe, HeroId(2)"
            plain info() -> ApiInfo = $crate::endpoints::info::GetInfo;
                "Ingestion throughput and table sizes of the server."
                ""
            plain health() -> Health = $crate::endpoints::info::GetHealth;
                "Which of the server's backing services answer."
                ""
            plain heroes(query: &HeroesQuery) -> Vec<Hero> = $crate::endpoints::assets::ListHeroes { query: query.clone() };
                "Every hero the game client describes, about 1.9 MB uncompressed."
                "&HeroesQuery::default()"
            plain hero(id: HeroId, query: &AssetsQuery) -> Hero = $crate::endpoints::assets::GetHero { id, query: query.clone() };
                "One hero by id."
                "HeroId(1), &AssetsQuery::default()"
            plain hero_by_name(name: &str, query: &AssetsQuery) -> Hero = $crate::endpoints::assets::GetHeroByName { name: name.to_owned(), query: query.clone() };
                "One hero by name. The name is sent as one percent-encoded path segment; an empty name, `.`, `..` or a name containing `/` is refused with `Error::InvalidRequest`."
                "\"Lady Geist\", &AssetsQuery::default()"
            plain items(query: &AssetsQuery) -> Vec<AssetItem> = $crate::endpoints::assets::ListItems { query: query.clone() };
                "Every ability, upgrade and weapon, about 6.4 MB uncompressed: on a slow link enable the `gzip` feature or raise the timeout."
                "&AssetsQuery::default()"
            plain item(id: ItemId, query: &AssetsQuery) -> AssetItem = $crate::endpoints::assets::GetItem { id, query: query.clone() };
                "One item by id."
                "ItemId(1548066885), &AssetsQuery::default()"
            plain item_by_class_name(class_name: &str, query: &AssetsQuery) -> AssetItem = $crate::endpoints::assets::GetItemByClassName { class_name: class_name.to_owned(), query: query.clone() };
                "One item by class name. It shares its route with [`item`](Self::item): the server reads a class name of digits only as an id. The name is sent as one percent-encoded path segment; an empty name, `.`, `..` or a name containing `/` is refused with `Error::InvalidRequest`."
                "\"upgrade_clip_size\", &AssetsQuery::default()"
            plain hero_items(hero: HeroId, query: &AssetsQuery) -> Vec<AssetItem> = $crate::endpoints::assets::ListHeroItems { hero, query: query.clone() };
                "The items of one hero."
                "HeroId(1), &AssetsQuery::default()"
            plain items_by_slot(slot: ItemSlot, query: &AssetsQuery) -> Vec<AssetItem> = $crate::endpoints::assets::ListItemsBySlot { slot, query: query.clone() };
                "The shop items of one column."
                "ItemSlot::Spirit, &AssetsQuery::default()"
            plain items_by_kind(kind: ItemKind, query: &AssetsQuery) -> Vec<AssetItem> = $crate::endpoints::assets::ListItemsByKind { kind, query: query.clone() };
                "The items of one kind."
                "ItemKind::Weapon, &AssetsQuery::default()"
            plain ranks(query: &AssetsQuery) -> Vec<Rank> = $crate::endpoints::assets::ListRanks { query: query.clone() };
                "Every rank tier with its name, colour and badge images."
                "&AssetsQuery::default()"
            plain rank(tier: u32, query: &AssetsQuery) -> Rank = $crate::endpoints::assets::GetRank { tier, query: query.clone() };
                "One rank tier; tier 0 is the unranked `Obscurus`."
                "1, &AssetsQuery::default()"
            plain patches() -> Vec<Patch> = $crate::endpoints::patches::ListPatches;
                "Patch notes from the forum's changelog feed, newest first."
                ""
            plain patch_feed() -> Vec<FeedItem> = $crate::endpoints::patches::ListPatchFeed;
                "Patch notes from the forum and Steam news together."
                ""
            plain patch_big_days() -> Vec<String> = $crate::endpoints::patches::ListBigDays;
                "The dates of the big patches, RFC 3339 strings as the server sends them."
                ""
        }
    };
    (@sync $($kind:ident $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal)*) => {
        $(
            $crate::endpoints::endpoint_methods!(@one_sync $kind $name($($arg: $ty),*) -> $out = $ctor; $summary $example);
        )*
    };
    (@async $($kind:ident $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal)*) => {
        $(
            $crate::endpoints::endpoint_methods!(@one_async $kind $name($($arg: $ty),*) -> $out = $ctor; $summary $example);
        )*
    };
    (@one_sync plain $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $($arg: $ty),*) -> $crate::Result<$out> {
            self.send(&$ctor)
        }
    };
    (@one_sync list $name:ident($ids:ident: $ty:ty $(, $arg:ident: $aty:ty)*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "An empty `ids` returns an empty `Vec` without a request."]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $ids: $ty $(, $arg: $aty)*) -> $crate::Result<$out> {
            if $ids.is_empty() {
                return Ok(Vec::new());
            }
            self.send(&$ctor)
        }
    };
    (@one_sync optional $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $($arg: $ty),*) -> $crate::Result<Option<$out>> {
            $crate::endpoints::optional(self.send(&$ctor))
        }
    };
    (@one_async plain $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $($arg: $ty),*) -> $crate::Result<$out> {
            self.send(&$ctor).await
        }
    };
    (@one_async list $name:ident($ids:ident: $ty:ty $(, $arg:ident: $aty:ty)*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "An empty `ids` returns an empty `Vec` without a request."]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $ids: $ty $(, $arg: $aty)*) -> $crate::Result<$out> {
            if $ids.is_empty() {
                return Ok(Vec::new());
            }
            self.send(&$ctor).await
        }
    };
    (@one_async optional $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = $crate::endpoints::errors_doc!()]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\nuse deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $($arg: $ty),*) -> $crate::Result<Option<$out>> {
            $crate::endpoints::optional(self.send(&$ctor).await)
        }
    };
}
pub(crate) use endpoint_methods;

/// Turns the server's 400 "cannot be live" answer into `Ok(None)`; any other
/// 400 (the route's parameter-validation answer) stays an error. The message
/// text is the one recorded in `tests/fixtures/spectate_url_400.json`.
pub(crate) fn optional<T>(result: crate::Result<T>) -> crate::Result<Option<T>> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(crate::Error::Status {
            status: 400,
            message: Some(ref m),
            ..
        }) if m.contains("cannot be live") => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::{Endpoint, to_request};
    use crate::{
        AccountId, ActiveMatchesQuery, ConfigBuilder, EnemyStatsQuery, HeroId, HeroStatsQuery,
        MatchFilter, MatchId, MateStatsQuery, Region, SteamSearchQuery,
    };

    fn url<E: Endpoint>(e: &E) -> String {
        to_request(
            e,
            &ConfigBuilder::default()
                .base_url("https://h")
                .build()
                .unwrap(),
        )
        .unwrap()
        .uri()
        .to_string()
    }

    #[test]
    fn paths_and_cache_keys() {
        assert_eq!(
            url(&matches::GetMatchMetadata { id: MatchId(42) }),
            "https://h/v1/matches/42/metadata"
        );
        assert_eq!(
            matches::GetMatchMetadata { id: MatchId(42) }
                .cache_key()
                .as_deref(),
            Some("matches/42/metadata.json")
        );
        assert_eq!(
            url(&matches::GetMatchSalts { id: MatchId(42) }),
            "https://h/v1/matches/42/salts"
        );
        assert_eq!(matches::GetMatchSalts { id: MatchId(42) }.cache_key(), None);
        assert_eq!(
            url(&matches::GetSpectateUrl { id: MatchId(42) }),
            "https://h/v1/matches/42/live/url"
        );
        assert_eq!(
            url(&matches::ListRecentlyFetched),
            "https://h/v1/matches/recently-fetched"
        );
        assert_eq!(
            url(&players::GetMatchHistory {
                account: AccountId(7)
            }),
            "https://h/v1/players/7/match-history"
        );
        assert_eq!(
            url(&players::GetPlayerRank {
                account: AccountId(7)
            }),
            "https://h/v1/players/7/rank"
        );
        assert_eq!(
            url(&players::GetMateStats {
                account: AccountId(7),
                query: MateStatsQuery::default()
            }),
            "https://h/v1/players/7/mate-stats"
        );
        assert_eq!(
            url(&players::GetEnemyStats {
                account: AccountId(7),
                query: EnemyStatsQuery::default()
            }),
            "https://h/v1/players/7/enemy-stats"
        );
        assert_eq!(
            url(&leaderboard::GetLeaderboard {
                region: Region::Europe
            }),
            "https://h/v1/leaderboard/Europe"
        );
        assert_eq!(
            url(&leaderboard::GetHeroLeaderboard {
                region: Region::NAmerica,
                hero: HeroId(2)
            }),
            "https://h/v1/leaderboard/NAmerica/2"
        );
        assert_eq!(url(&info::GetInfo), "https://h/v1/info");
        assert_eq!(url(&info::GetHealth), "https://h/v1/info/health");
    }

    #[test]
    fn stats_filters_follow_openapi_names() {
        let ids = vec![AccountId(1)];
        let q = HeroStatsQuery {
            hero_ids: vec![HeroId(2), HeroId(3)],
            game_mode: Some("street_brawl".into()),
            min_networth: Some(10),
            ..HeroStatsQuery::default()
        };
        assert_eq!(
            url(&players::GetHeroStats {
                ids: ids.clone(),
                query: q
            }),
            "https://h/v1/players/hero-stats?account_ids=1&game_mode=street_brawl&hero_ids=2%2C3&min_networth=10"
        );
        assert_eq!(
            url(&players::GetHeroStats {
                ids,
                query: HeroStatsQuery::default()
            }),
            "https://h/v1/players/hero-stats?account_ids=1"
        );
        let q = MateStatsQuery {
            same_party: Some(true),
            min_matches_played: Some(5),
            ..MateStatsQuery::default()
        };
        assert_eq!(
            url(&players::GetMateStats {
                account: AccountId(7),
                query: q
            }),
            "https://h/v1/players/7/mate-stats?min_matches_played=5&same_party=true"
        );
        let q = EnemyStatsQuery {
            max_match_id: Some(MatchId(99)),
            ..EnemyStatsQuery::default()
        };
        assert_eq!(
            url(&players::GetEnemyStats {
                account: AccountId(7),
                query: q
            }),
            "https://h/v1/players/7/enemy-stats?max_match_id=99"
        );
    }

    #[test]
    fn id_lists_are_comma_joined() {
        let ids = vec![AccountId(1), AccountId(22)];
        assert_eq!(
            url(&players::GetPlayerRanks { ids: ids.clone() }),
            "https://h/v1/players/rank?account_ids=1%2C22"
        );
        assert_eq!(
            url(&players::GetHeroStats {
                ids: ids.clone(),
                query: HeroStatsQuery::default()
            }),
            "https://h/v1/players/hero-stats?account_ids=1%2C22"
        );
        assert_eq!(
            url(&steam::GetSteamProfiles { ids }),
            "https://h/v1/players/steam?account_ids=1%2C22"
        );
    }

    #[test]
    fn filters_omit_none_and_keep_openapi_names() {
        let empty = players::GetRankDistribution {
            filter: MatchFilter::default(),
        };
        assert_eq!(url(&empty), "https://h/v1/players/rank/distribution");
        let filter = MatchFilter {
            min_unix_timestamp: Some(1_790_000_000),
            max_duration_s: Some(3600),
            is_low_pri_pool: Some(false),
            ..MatchFilter::default()
        };
        assert_eq!(
            url(&players::GetRankDistribution { filter }),
            "https://h/v1/players/rank/distribution?min_unix_timestamp=1790000000&max_duration_s=3600&is_low_pri_pool=false"
        );
        let active = ActiveMatchesQuery {
            account_ids: vec![AccountId(5), AccountId(6)],
            ..ActiveMatchesQuery::default()
        };
        assert_eq!(
            url(&matches::ListActiveMatches { query: active }),
            "https://h/v1/matches/active?account_ids=5%2C6"
        );
        let search = steam::SearchSteam {
            search_query: "x y".into(),
            query: SteamSearchQuery {
                limit: Some(3),
                ..SteamSearchQuery::default()
            },
        };
        assert_eq!(
            url(&search),
            "https://h/v1/players/steam-search?search_query=x+y&limit=3"
        );
    }

    fn rejected<E: Endpoint>(e: &E) -> String {
        let cfg = ConfigBuilder::default()
            .base_url("https://h")
            .build()
            .unwrap();
        match to_request(e, &cfg) {
            Err(crate::Error::InvalidRequest { message, .. }) => message,
            other => panic!("{:?}", other.map(|r| r.uri().to_string())),
        }
    }

    #[test]
    fn assets_and_patches_paths_and_queries() {
        use crate::{AssetsQuery, HeroesQuery, ItemId, ItemKind, ItemSlot, Language};
        let q = AssetsQuery::default();
        assert_eq!(
            url(&assets::ListHeroes {
                query: HeroesQuery::default()
            }),
            "https://h/v1/assets/heroes"
        );
        let full = HeroesQuery {
            assets: AssetsQuery {
                language: Some(Language::Russian),
                client_version: Some(6027),
            },
            only_active: Some(true),
        };
        assert_eq!(
            url(&assets::ListHeroes { query: full }),
            "https://h/v1/assets/heroes?language=russian&client_version=6027&only_active=true"
        );
        assert_eq!(
            url(&assets::GetHero {
                id: HeroId(1),
                query: q.clone()
            }),
            "https://h/v1/assets/heroes/1"
        );
        assert_eq!(
            url(&assets::GetHeroByName {
                name: "Lady Geist".into(),
                query: q.clone()
            }),
            "https://h/v1/assets/heroes/by-name/Lady%20Geist"
        );
        assert_eq!(
            url(&assets::GetHeroByName {
                name: "Mo & Krill?#%".into(),
                query: q.clone()
            }),
            "https://h/v1/assets/heroes/by-name/Mo%20%26%20Krill%3F%23%25"
        );
        assert_eq!(
            url(&assets::GetHeroByName {
                name: "Призрак".into(),
                query: q.clone()
            }),
            "https://h/v1/assets/heroes/by-name/%D0%9F%D1%80%D0%B8%D0%B7%D1%80%D0%B0%D0%BA"
        );
        assert_eq!(
            url(&assets::ListItems { query: q.clone() }),
            "https://h/v1/assets/items"
        );
        assert_eq!(
            url(&assets::GetItem {
                id: ItemId(1548066885),
                query: q.clone()
            }),
            "https://h/v1/assets/items/1548066885"
        );
        assert_eq!(
            url(&assets::GetItemByClassName {
                class_name: "upgrade_clip_size".into(),
                query: q.clone()
            }),
            "https://h/v1/assets/items/upgrade_clip_size"
        );
        assert_eq!(
            url(&assets::ListHeroItems {
                hero: HeroId(7),
                query: q.clone()
            }),
            "https://h/v1/assets/items/by-hero-id/7"
        );
        assert_eq!(
            url(&assets::ListItemsBySlot {
                slot: ItemSlot::Vitality,
                query: q.clone()
            }),
            "https://h/v1/assets/items/by-slot-type/vitality"
        );
        assert_eq!(
            url(&assets::ListItemsByKind {
                kind: ItemKind::Upgrade,
                query: q.clone()
            }),
            "https://h/v1/assets/items/by-type/upgrade"
        );
        assert_eq!(
            url(&assets::ListRanks { query: q.clone() }),
            "https://h/v1/assets/ranks"
        );
        assert_eq!(
            url(&assets::GetRank {
                tier: 11,
                query: q.clone()
            }),
            "https://h/v1/assets/ranks/11"
        );
        assert_eq!(url(&patches::ListPatches), "https://h/v1/patches");
        assert_eq!(url(&patches::ListPatchFeed), "https://h/v2/patches");
        assert_eq!(url(&patches::ListBigDays), "https://h/v1/patches/big-days");
    }

    #[test]
    fn bad_segments_and_versions_are_refused_before_sending() {
        use crate::{AssetsQuery, ItemKind};
        let q = AssetsQuery::default();
        for name in ["", ".", "..", "a/b"] {
            let message = rejected(&assets::GetHeroByName {
                name: name.into(),
                query: q.clone(),
            });
            assert!(message.starts_with("the hero name "), "{name:?}: {message}");
            rejected(&assets::GetItemByClassName {
                class_name: name.into(),
                query: q.clone(),
            });
        }
        rejected(&assets::ListItemsByKind {
            kind: ItemKind::Unknown("../x".into()),
            query: q.clone(),
        });
        let too_new = AssetsQuery {
            client_version: Some(u32::try_from(i32::MAX).unwrap() + 1),
            ..AssetsQuery::default()
        };
        let message = rejected(&assets::ListItems { query: too_new });
        assert!(message.contains("int32"), "{message}");
        let newest = AssetsQuery {
            client_version: Some(u32::try_from(i32::MAX).unwrap()),
            ..AssetsQuery::default()
        };
        assert!(url(&assets::ListItems { query: newest }).ends_with("client_version=2147483647"));
    }
}
