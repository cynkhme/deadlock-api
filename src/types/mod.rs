//! Response models and query structs, one module per endpoint group. The
//! module is private: every type is re-exported at the crate root, its one
//! public path, so the files here can be regrouped without a breaking
//! release.
//!
//! ```compile_fail,E0603
//! use deadlock_api::types::matches::MatchSalts;
//! ```

pub mod filters;
pub mod info;
pub mod leaderboard;
pub mod match_metadata;
pub mod matches;
pub mod players;
pub mod steam;
