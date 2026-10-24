//! Steam profile routes.

use serde::{Deserialize, Serialize};

use crate::AccountId;

/// One entry of `GET /v1/players/steam` and `GET /v1/players/steam-search`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SteamProfile {
    /// The Steam account id.
    pub account_id: AccountId,
    /// The display name.
    #[serde(default)]
    pub personaname: Option<String>,
    /// The Steam profile URL.
    #[serde(default)]
    pub profileurl: Option<String>,
    /// Small avatar URL.
    #[serde(default)]
    pub avatar: Option<String>,
    /// Medium avatar URL.
    #[serde(default)]
    pub avatarmedium: Option<String>,
    /// Full avatar URL.
    #[serde(default)]
    pub avatarfull: Option<String>,
    /// The real name, when the profile shows one.
    #[serde(default)]
    pub realname: Option<String>,
    /// Two-letter country code.
    #[serde(default)]
    pub countrycode: Option<String>,
    /// When the server last refreshed the profile, as an RFC 3339 string.
    #[serde(default)]
    pub last_updated: Option<String>,
    /// Steam friends with a Deadlock profile.
    #[serde(default)]
    pub friends: Vec<SteamFriend>,
    /// Matches played in the last 30 days.
    #[serde(default)]
    pub matches_played_last_30d: Option<u32>,
    /// Average badge of the player's last team.
    #[serde(default)]
    pub last_team_avg_badge: Option<u32>,
}

/// A friend in [`SteamProfile::friends`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SteamFriend {
    /// The friend's Steam account id.
    pub account_id: AccountId,
    /// When they became friends, as an RFC 3339 string.
    #[serde(default)]
    pub friend_since: Option<String>,
}
