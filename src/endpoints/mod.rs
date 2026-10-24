//! One struct per route, each implementing [`Endpoint`](crate::endpoint::Endpoint),
//! and the macro that gives both clients the same convenience methods.

pub(crate) mod info;
pub(crate) mod leaderboard;
pub(crate) mod matches;
pub(crate) mod players;
pub(crate) mod steam;

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
            list hero_stats(ids: &[AccountId]) -> Vec<HeroStats> = $crate::endpoints::players::GetHeroStats { ids: ids.to_vec() };
                "Per-hero statistics of several players."
                "&[AccountId(167576350)]"
            plain mate_stats(account: AccountId) -> Vec<MateStats> = $crate::endpoints::players::GetMateStats { account };
                "A player's results with each teammate."
                "AccountId(167576350)"
            plain enemy_stats(account: AccountId) -> Vec<EnemyStats> = $crate::endpoints::players::GetEnemyStats { account };
                "A player's results against each opponent."
                "AccountId(167576350)"
            list steam_profiles(ids: &[AccountId]) -> Vec<SteamProfile> = $crate::endpoints::steam::GetSteamProfiles { ids: ids.to_vec() };
                "Steam profiles (names, avatars, friends) of several players."
                "&[AccountId(167576350)]"
            plain steam_search(search_query: &str, query: &SteamSearchQuery) -> Vec<SteamProfile> = $crate::endpoints::steam::SearchSteam { search_query: search_query.to_owned(), query: query.clone() };
                "Steam profiles whose name matches `search_query`."
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
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $($arg: $ty),*) -> $crate::Result<$out> {
            self.send(&$ctor)
        }
    };
    (@one_sync list $name:ident($ids:ident: $ty:ty) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "An empty `ids` returns an empty `Vec` without a request."]
        #[doc = ""]
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $ids: $ty) -> $crate::Result<$out> {
            if $ids.is_empty() {
                return Ok(Vec::new());
            }
            self.send(&$ctor)
        }
    };
    (@one_sync optional $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# fn main() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::blocking::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ")?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub fn $name(&self, $($arg: $ty),*) -> $crate::Result<Option<$out>> {
            $crate::endpoints::optional(self.send(&$ctor))
        }
    };
    (@one_async plain $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $($arg: $ty),*) -> $crate::Result<$out> {
            self.send(&$ctor).await
        }
    };
    (@one_async list $name:ident($ids:ident: $ty:ty) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "An empty `ids` returns an empty `Vec` without a request."]
        #[doc = ""]
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $ids: $ty) -> $crate::Result<$out> {
            if $ids.is_empty() {
                return Ok(Vec::new());
            }
            self.send(&$ctor).await
        }
    };
    (@one_async optional $name:ident($($arg:ident: $ty:ty),*) -> $out:ty = $ctor:expr; $summary:literal $example:literal) => {
        #[doc = $summary]
        #[doc = ""]
        #[doc = "# Errors\n\n[`Error::NotFound`](crate::Error::NotFound) when the server has no data for the id; [`Error::RateLimited`](crate::Error::RateLimited) and [`Error::Overloaded`](crate::Error::Overloaded) on 429 and 503, with the server's `Retry-After` when it sent one; [`Error::Status`](crate::Error::Status) on any other non-2xx; [`Error::Transport`](crate::Error::Transport) when no response arrived; [`Error::Decode`](crate::Error::Decode) when the body is not the expected JSON."]
        #[doc = ""]
        #[doc = concat!("# Examples\n\n```no_run\n# async fn run() -> deadlock_api::Result<()> {\n# use deadlock_api::*;\nlet api = deadlock_api::Client::new()?;\nlet value = api.", stringify!($name), "(", $example, ").await?;\n# let _ = value;\n# Ok(()) }\n```")]
        pub async fn $name(&self, $($arg: $ty),*) -> $crate::Result<Option<$out>> {
            $crate::endpoints::optional(self.send(&$ctor).await)
        }
    };
}
pub(crate) use endpoint_methods;

/// Turns the server's 400 "cannot be live" answer into `Ok(None)`.
pub(crate) fn optional<T>(result: crate::Result<T>) -> crate::Result<Option<T>> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(crate::Error::Status {
            status: 400,
            message: Some(_),
            ..
        }) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::{Endpoint, to_request};
    use crate::{
        AccountId, ActiveMatchesQuery, ConfigBuilder, HeroId, MatchFilter, MatchId, Region,
        SteamSearchQuery,
    };

    fn url<E: Endpoint>(e: &E) -> String {
        to_request(
            e,
            &ConfigBuilder::default()
                .base_url("https://h")
                .build()
                .unwrap(),
        )
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
                account: AccountId(7)
            }),
            "https://h/v1/players/7/mate-stats"
        );
        assert_eq!(
            url(&players::GetEnemyStats {
                account: AccountId(7)
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
    fn id_lists_are_comma_joined() {
        let ids = vec![AccountId(1), AccountId(22)];
        assert_eq!(
            url(&players::GetPlayerRanks { ids: ids.clone() }),
            "https://h/v1/players/rank?account_ids=1%2C22"
        );
        assert_eq!(
            url(&players::GetHeroStats { ids: ids.clone() }),
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
}
