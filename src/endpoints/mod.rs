//! One struct per route, each implementing [`Endpoint`](crate::endpoint::Endpoint).
#![allow(dead_code)] // used by the blocking and async clients

pub(crate) mod info;
pub(crate) mod leaderboard;
pub(crate) mod matches;
pub(crate) mod players;
pub(crate) mod steam;

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
