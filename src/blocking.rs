//! The synchronous client, on [ureq](https://docs.rs/ureq).

use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response};
use serde::de::DeserializeOwned;

use crate::endpoint::{Endpoint, execute_raw, raw_request, raw_url, run};
use crate::retry::copy_request;
use crate::{
    AccountId, AccountRank, ActiveMatch, ActiveMatchesQuery, ApiInfo, Config, ConfigBuilder,
    EnemyStats, EnemyStatsQuery, Error, Health, HeroId, HeroStats, HeroStatsQuery, Leaderboard,
    MatchFilter, MatchHistoryEntry, MatchId, MatchMetadata, MatchSalts, MateStats, MateStatsQuery,
    RankDistributionEntry, RankResponse, RecentMatch, Region, Result, RetryPolicy, SpectateUrl,
    SteamProfile, SteamSearchQuery,
};

#[cfg(feature = "rustls")]
const TLS_PROVIDER: ureq::tls::TlsProvider = ureq::tls::TlsProvider::Rustls;
#[cfg(all(feature = "native-tls", not(feature = "rustls")))]
const TLS_PROVIDER: ureq::tls::TlsProvider = ureq::tls::TlsProvider::NativeTls;
#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!("deadlock-api: the `blocking` feature needs `rustls` or `native-tls`");

/// A blocking client: one connection pool, one [`Config`], one optional cache.
///
/// Cheap to clone (the pool is shared). Build it with [`Client::builder`] or
/// [`Client::new`] for the defaults. Redirects are not followed: a 3xx comes
/// back as [`Error::Status`], so the API key is never re-sent to another host.
///
/// # Examples
///
/// ```no_run
/// use deadlock_api::blocking::Client;
/// use deadlock_api::MatchId;
///
/// # fn main() -> deadlock_api::Result<()> {
/// let api = Client::builder()
///     .api_key(std::env::var("DEADLOCK_API_KEY").ok())
///     .build()?;
/// let meta = api.match_metadata(MatchId(122539002))?;
/// println!("{} players", meta.match_info.players.len());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    agent: ureq::Agent,
    config: Arc<Config>,
    retry: Option<RetryPolicy>,
}

/// Builder for [`Client`]; what it sets, apart from the retry policy, is
/// readable afterwards through [`Client::config`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    config: ConfigBuilder,
    retry: Option<RetryPolicy>,
}

builder_setters!(ClientBuilder);

impl ClientBuilder {
    /// Validates the settings and builds the client.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] when the base URL has no scheme or host, or the user
    /// agent or API key is not a valid header value.
    pub fn build(self) -> Result<Client> {
        let config = self.config.build()?;
        // ureq 3 defaults to RootCerts::WebPki (bundled roots), and with
        // native-tls it also switches the OS roots off; reqwest trusts the
        // OS store.
        let tls = ureq::tls::TlsConfig::builder()
            .provider(TLS_PROVIDER)
            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
            .build();
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(config.connect_timeout()))
            .timeout_global(Some(config.timeout()))
            .user_agent(config.user_agent())
            .tls_config(tls)
            .build()
            .new_agent();
        Ok(Client {
            agent,
            config: Arc::new(config),
            retry: self.retry,
        })
    }
}

impl Client {
    /// A builder with the default settings.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// A client with the default settings and no API key.
    ///
    /// # Errors
    ///
    /// Never in practice: the defaults are valid; the `Result` mirrors
    /// [`ClientBuilder::build`].
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// The settings this client was built with.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Any route under the server root, decoded as `T`.
    ///
    /// The escape hatch for routes without a typed method: `path` starts
    /// with `/`, `query` pairs are form-encoded. Nothing is cached. The API
    /// key is sent as `X-API-KEY` from the config; a query pair named
    /// `api_key` is refused.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] for an `api_key` pair, [`Error::InvalidRequest`] for
    /// a path that does not start with `/`, carries a `?` or `#`, or does not
    /// form a URL; otherwise as for every request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> deadlock_api::Result<()> {
    /// let api = deadlock_api::blocking::Client::new()?;
    /// let v: serde_json::Value =
    ///     api.get_json("/v1/analytics/hero-stats", &[("min_unix_timestamp", "1790000000")])?;
    /// # let _ = v;
    /// # Ok(()) }
    /// ```
    pub fn get_json<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        let url = raw_url(&self.config, path, query)?;
        let body = self.get_raw_at(&url)?;
        serde_json::from_slice(&body).map_err(|source| Error::Decode {
            location: url,
            source,
        })
    }

    /// Any route under the server root, as raw bytes.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] for an `api_key` pair, [`Error::InvalidRequest`] for
    /// a path that does not start with `/`, carries a `?` or `#`, or does not
    /// form a URL; otherwise as for every request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> deadlock_api::Result<()> {
    /// let api = deadlock_api::blocking::Client::new()?;
    /// let bytes = api.get_raw("/v1/info", &[])?;
    /// # let _ = bytes;
    /// # Ok(()) }
    /// ```
    pub fn get_raw(&self, path: &str, query: &[(&str, &str)]) -> Result<Bytes> {
        self.get_raw_at(&raw_url(&self.config, path, query)?)
    }

    fn get_raw_at(&self, url: &str) -> Result<Bytes> {
        let resp = self.transport(raw_request(&self.config, url)?)?;
        execute_raw(url, resp)
    }

    fn send<E: Endpoint>(&self, endpoint: &E) -> Result<E::Output> {
        run(endpoint, &self.config, |req| self.transport(req))
    }

    fn transport(&self, req: Request<()>) -> Result<Response<Bytes>> {
        let Some(policy) = &self.retry else {
            return self.send_once(req);
        };
        let mut attempts = 1;
        loop {
            let result = self.send_once(copy_request(&req));
            let Some(pause) = policy.pause(attempts, &result, unsent) else {
                return result;
            };
            log::info!("retrying {} in {pause:?}", req.uri());
            std::thread::sleep(pause);
            attempts += 1;
        }
    }

    fn send_once(&self, req: Request<()>) -> Result<Response<Bytes>> {
        let url = req.uri().to_string();
        let mut resp = self.agent.run(req).map_err(|e| Error::Transport {
            url: url.clone(),
            source: Box::new(e),
        })?;
        let body = resp
            .body_mut()
            .read_to_vec()
            .map_err(|e| Error::Transport {
                url,
                source: Box::new(e),
            })?;
        Ok(resp.map(|_| Bytes::from(body)))
    }

    crate::endpoints::endpoint_methods!(sync);
}

/// Whether ureq failed before the request reached the server.
fn unsent(error: &Error) -> bool {
    let Error::Transport { source, .. } = error else {
        return false;
    };
    match source.downcast_ref::<ureq::Error>() {
        Some(
            ureq::Error::HostNotFound
            | ureq::Error::ConnectionFailed
            | ureq::Error::Timeout(ureq::Timeout::Resolve | ureq::Timeout::Connect),
        ) => true,
        Some(ureq::Error::Io(e)) => matches!(
            e.kind(),
            std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::HostUnreachable
                | std::io::ErrorKind::NetworkUnreachable
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_trusts_the_platform_store_like_the_async_client() {
        let api = Client::new().unwrap();
        let roots = api.agent.config().tls_config().root_certs();
        assert!(
            matches!(roots, ureq::tls::RootCerts::PlatformVerifier),
            "{roots:?}"
        );
    }

    #[test]
    fn only_failures_before_sending_count_as_unsent() {
        let wrap = |e: ureq::Error| Error::Transport {
            url: "http://h/".into(),
            source: Box::new(e),
        };
        let io = |kind: std::io::ErrorKind| ureq::Error::Io(kind.into());
        for e in [
            ureq::Error::HostNotFound,
            ureq::Error::ConnectionFailed,
            ureq::Error::Timeout(ureq::Timeout::Resolve),
            ureq::Error::Timeout(ureq::Timeout::Connect),
            io(std::io::ErrorKind::ConnectionRefused),
            io(std::io::ErrorKind::HostUnreachable),
            io(std::io::ErrorKind::NetworkUnreachable),
        ] {
            assert!(unsent(&wrap(e)));
        }
        for e in [
            ureq::Error::Timeout(ureq::Timeout::Global),
            ureq::Error::Timeout(ureq::Timeout::RecvResponse),
            io(std::io::ErrorKind::ConnectionReset),
            io(std::io::ErrorKind::TimedOut),
        ] {
            assert!(!unsent(&wrap(e)));
        }
        assert!(!unsent(&Error::Transport {
            url: "http://h/".into(),
            source: "not ureq".into(),
        }));
    }
}
