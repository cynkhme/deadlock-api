//! Player routes.

use crate::AccountId;
use crate::endpoint::Endpoint;
use crate::types::filters::{
    EnemyStatsQuery, HeroStatsQuery, MatchFilter, MateStatsQuery, join_ids,
};
use crate::types::players::{
    AccountRank, EnemyStats, HeroStats, MatchHistoryEntry, MateStats, RankDistributionEntry,
    RankResponse,
};

/// `GET /v1/players/{id}/match-history`.
pub(crate) struct GetMatchHistory {
    pub account: AccountId,
}

impl Endpoint for GetMatchHistory {
    type Output = Vec<MatchHistoryEntry>;
    fn path(&self) -> String {
        format!("/v1/players/{}/match-history", self.account)
    }
}

/// `GET /v1/players/{id}/rank`.
pub(crate) struct GetPlayerRank {
    pub account: AccountId,
}

impl Endpoint for GetPlayerRank {
    type Output = RankResponse;
    fn path(&self) -> String {
        format!("/v1/players/{}/rank", self.account)
    }
}

/// `GET /v1/players/rank?account_ids=…`.
pub(crate) struct GetPlayerRanks {
    pub ids: Vec<AccountId>,
}

impl Endpoint for GetPlayerRanks {
    type Output = Vec<AccountRank>;
    fn path(&self) -> String {
        "/v1/players/rank".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        vec![("account_ids", join_ids(&self.ids))]
    }
}

/// `GET /v1/players/rank/distribution`.
pub(crate) struct GetRankDistribution {
    pub filter: MatchFilter,
}

impl Endpoint for GetRankDistribution {
    type Output = Vec<RankDistributionEntry>;
    fn path(&self) -> String {
        "/v1/players/rank/distribution".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        self.filter.query_pairs()
    }
}

/// `GET /v1/players/hero-stats?account_ids=…`.
pub(crate) struct GetHeroStats {
    pub ids: Vec<AccountId>,
    pub query: HeroStatsQuery,
}

impl Endpoint for GetHeroStats {
    type Output = Vec<HeroStats>;
    fn path(&self) -> String {
        "/v1/players/hero-stats".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut q = vec![("account_ids", join_ids(&self.ids))];
        q.extend(self.query.query_pairs());
        q
    }
}

/// `GET /v1/players/{id}/mate-stats`.
pub(crate) struct GetMateStats {
    pub account: AccountId,
    pub query: MateStatsQuery,
}

impl Endpoint for GetMateStats {
    type Output = Vec<MateStats>;
    fn path(&self) -> String {
        format!("/v1/players/{}/mate-stats", self.account)
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        self.query.query_pairs()
    }
}

/// `GET /v1/players/{id}/enemy-stats`.
pub(crate) struct GetEnemyStats {
    pub account: AccountId,
    pub query: EnemyStatsQuery,
}

impl Endpoint for GetEnemyStats {
    type Output = Vec<EnemyStats>;
    fn path(&self) -> String {
        format!("/v1/players/{}/enemy-stats", self.account)
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        self.query.query_pairs()
    }
}
