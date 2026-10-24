//! Info routes.

use crate::endpoint::Endpoint;
use crate::types::info::{ApiInfo, Health};

/// `GET /v1/info`.
pub(crate) struct GetInfo;

impl Endpoint for GetInfo {
    type Output = ApiInfo;
    fn path(&self) -> String {
        "/v1/info".into()
    }
}

/// `GET /v1/info/health`.
pub(crate) struct GetHealth;

impl Endpoint for GetHealth {
    type Output = Health;
    fn path(&self) -> String {
        "/v1/info/health".into()
    }
}
