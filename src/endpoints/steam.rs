//! Steam profile routes.

use crate::AccountId;
use crate::endpoint::Endpoint;
use crate::types::filters::{SteamSearchQuery, join_ids};
use crate::types::steam::SteamProfile;

/// `GET /v1/players/steam?account_ids=…`.
pub(crate) struct GetSteamProfiles {
    pub ids: Vec<AccountId>,
}

impl Endpoint for GetSteamProfiles {
    type Output = Vec<SteamProfile>;
    fn path(&self) -> String {
        "/v1/players/steam".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        vec![("account_ids", join_ids(&self.ids))]
    }
}

/// `GET /v1/players/steam-search?search_query=…`.
pub(crate) struct SearchSteam {
    pub search_query: String,
    pub query: SteamSearchQuery,
}

impl Endpoint for SearchSteam {
    type Output = Vec<SteamProfile>;
    fn path(&self) -> String {
        "/v1/players/steam-search".into()
    }
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut q = vec![("search_query", self.search_query.clone())];
        q.extend(self.query.query_pairs());
        q
    }
}
