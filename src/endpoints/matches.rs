//! Match routes.

use crate::endpoint::Endpoint;
use crate::types::filters::ActiveMatchesQuery;
use crate::types::matches::{ActiveMatch, MatchSalts, RecentMatch, SpectateUrl};
use crate::{MatchId, MatchMetadata};

/// `GET /v1/matches/{id}/metadata`; the one cached route.
pub(crate) struct GetMatchMetadata {
    pub id: MatchId,
}

impl Endpoint for GetMatchMetadata {
    type Output = MatchMetadata;
    fn path(&self) -> String {
        format!("/v1/matches/{}/metadata", self.id)
    }
    fn cache_key(&self) -> Option<String> {
        Some(format!("matches/{}/metadata.json", self.id))
    }
}

/// `GET /v1/matches/{id}/salts`.
pub(crate) struct GetMatchSalts {
    pub id: MatchId,
}

impl Endpoint for GetMatchSalts {
    type Output = MatchSalts;
    fn path(&self) -> String {
        format!("/v1/matches/{}/salts", self.id)
    }
}

/// `GET /v1/matches/{id}/live/url`.
pub(crate) struct GetSpectateUrl {
    pub id: MatchId,
}

impl Endpoint for GetSpectateUrl {
    type Output = SpectateUrl;
    fn path(&self) -> String {
        format!("/v1/matches/{}/live/url", self.id)
    }
}

/// `GET /v1/matches/active`.
pub(crate) struct ListActiveMatches {
    pub query: ActiveMatchesQuery,
}

impl Endpoint for ListActiveMatches {
    type Output = Vec<ActiveMatch>;
    fn path(&self) -> String {
        "/v1/matches/active".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        self.query.query_pairs()
    }
}

/// `GET /v1/matches/recently-fetched`.
pub(crate) struct ListRecentlyFetched;

impl Endpoint for ListRecentlyFetched {
    type Output = Vec<RecentMatch>;
    fn path(&self) -> String {
        "/v1/matches/recently-fetched".into()
    }
}
