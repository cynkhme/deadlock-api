//! Response models and query structs, one module per endpoint group. Field
//! names mirror the server's JSON one to one, so the API documentation
//! applies directly.

pub mod filters;
pub mod info;
pub mod leaderboard;
pub mod match_metadata;
pub mod matches;
pub mod players;
pub mod steam;
