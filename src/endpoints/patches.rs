//! Patch notes routes.

use crate::endpoint::Endpoint;
use crate::types::patches::{FeedItem, Patch};

/// `GET /v1/patches`.
pub(crate) struct ListPatches;

impl Endpoint for ListPatches {
    type Output = Vec<Patch>;
    fn path(&self) -> String {
        "/v1/patches".into()
    }
}

/// `GET /v2/patches`.
pub(crate) struct ListPatchFeed;

impl Endpoint for ListPatchFeed {
    type Output = Vec<FeedItem>;
    fn path(&self) -> String {
        "/v2/patches".into()
    }
}

/// `GET /v1/patches/big-days`.
pub(crate) struct ListBigDays;

impl Endpoint for ListBigDays {
    type Output = Vec<String>;
    fn path(&self) -> String {
        "/v1/patches/big-days".into()
    }
}
