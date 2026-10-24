//! Leaderboard routes.

use crate::HeroId;
use crate::endpoint::Endpoint;
use crate::types::leaderboard::{Leaderboard, Region};

/// `GET /v1/leaderboard/{region}`.
pub(crate) struct GetLeaderboard {
    pub region: Region,
}

impl Endpoint for GetLeaderboard {
    type Output = Leaderboard;
    fn path(&self) -> String {
        format!("/v1/leaderboard/{}", self.region.as_path_segment())
    }
}

/// `GET /v1/leaderboard/{region}/{hero_id}`.
pub(crate) struct GetHeroLeaderboard {
    pub region: Region,
    pub hero: HeroId,
}

impl Endpoint for GetHeroLeaderboard {
    type Output = Leaderboard;
    fn path(&self) -> String {
        format!(
            "/v1/leaderboard/{}/{}",
            self.region.as_path_segment(),
            self.hero
        )
    }
}
