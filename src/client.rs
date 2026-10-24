//! The asynchronous client, on [reqwest](https://docs.rs/reqwest) and tokio.

use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response};
use serde::de::DeserializeOwned;

use crate::endpoint::{
    Endpoint, cache_lookup, cache_store, decode_at, execute, execute_raw, is_json, raw_request,
    raw_url, to_request,
};
use crate::{
    AccountId, AccountRank, ActiveMatch, ActiveMatchesQuery, ApiInfo, Config, ConfigBuilder,
    EnemyStats, EnemyStatsQuery, Error, Health, HeroId, HeroStats, HeroStatsQuery, Leaderboard,
    MatchFilter, MatchHistoryEntry, MatchId, MatchMetadata, MatchSalts, MateStats, MateStatsQuery,
    RankDistributionEntry, RankResponse, RecentMatch, Region, Result, SpectateUrl, SteamProfile,
    SteamSearchQuery,
};

#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!("deadlock-api: the `async` feature needs `rustls` or `native-tls`");

/// An async client: one connection pool, one [`Config`], one optional cache.
///
/// Cheap to clone (the pool is shared) and `Send`, so it can move into
/// spawned tasks. The cache is synchronous by design; the client calls it on
/// tokio's blocking pool, so a slow disk never stalls a worker thread.
/// Redirects are not followed: a 3xx comes back as [`Error::Status`], so the
/// API key is never re-sent to another host.
///
/// # Examples
///
/// ```no_run
/// use deadlock_api::{Client, MatchId};
///
/// # async fn run() -> deadlock_api::Result<()> {
/// let api = Client::builder()
///     .api_key(std::env::var("DEADLOCK_API_KEY").ok())
///     .build()?;
/// let meta = api.match_metadata(MatchId(122539002)).await?;
/// println!("{} players", meta.match_info.players.len());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    config: Arc<Config>,
}

/// Builder for [`Client`]; the setters are those of [`ConfigBuilder`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    config: ConfigBuilder,
}

builder_setters!(ClientBuilder);

impl ClientBuilder {
    /// Validates the settings and builds the client.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] when the base URL has no scheme or host, or the user
    /// agent or API key is not a valid header value; [`Error::Transport`] when
    /// reqwest cannot initialise its TLS backend.
    pub fn build(self) -> Result<Client> {
        let config = self.config.build()?;
        let builder = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(config.user_agent())
            .connect_timeout(config.connect_timeout())
            .timeout(config.timeout());
        #[cfg(feature = "rustls")]
        let builder = builder.use_rustls_tls();
        #[cfg(all(feature = "native-tls", not(feature = "rustls")))]
        let builder = builder.use_native_tls();
        let http = builder.build().map_err(|e| Error::Transport {
            url: config.base_url().to_owned(),
            source: Box::new(e),
        })?;
        Ok(Client {
            http,
            config: Arc::new(config),
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
    /// As [`ClientBuilder::build`].
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
    /// with `/v1/`, `query` pairs are form-encoded. Nothing is cached. The
    /// API key is sent as `X-API-KEY` from the config; a query pair named
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
    /// # async fn run() -> deadlock_api::Result<()> {
    /// let api = deadlock_api::Client::new()?;
    /// let v: serde_json::Value = api
    ///     .get_json("/v1/analytics/hero-stats", &[("min_unix_timestamp", "1790000000")])
    ///     .await?;
    /// # let _ = v;
    /// # Ok(()) }
    /// ```
    pub async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let url = raw_url(&self.config, path, query)?;
        let body = self.get_raw_at(&url).await?;
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
    /// # async fn run() -> deadlock_api::Result<()> {
    /// let api = deadlock_api::Client::new()?;
    /// let bytes = api.get_raw("/v1/info", &[]).await?;
    /// # let _ = bytes;
    /// # Ok(()) }
    /// ```
    pub async fn get_raw(&self, path: &str, query: &[(&str, &str)]) -> Result<Bytes> {
        self.get_raw_at(&raw_url(&self.config, path, query)?).await
    }

    async fn get_raw_at(&self, url: &str) -> Result<Bytes> {
        let resp = self.transport(raw_request(&self.config, url)?).await?;
        execute_raw(url, resp)
    }

    /// The core's request flow around `await`: cache lookup, transport on a
    /// miss, cache store after a 2xx, decode. Cache calls run on the
    /// blocking pool; a cache task that panics counts as a miss.
    async fn send<E: Endpoint>(&self, endpoint: &E) -> Result<E::Output> {
        let key = endpoint
            .cache_key()
            .filter(|_| self.config.cache().is_some());
        if let Some(key) = &key {
            let (cfg, k) = (self.config.clone(), key.clone());
            let hit = tokio::task::spawn_blocking(move || cache_lookup(&cfg, &k))
                .await
                .unwrap_or_default();
            if let Some(body) = hit {
                if is_json(&body) {
                    return decode_at::<E>(key, &body);
                }
                log::warn!("cached body for {key} is not JSON, fetching instead");
            }
        }
        let req = to_request(endpoint, &self.config)?;
        let url = req.uri().to_string();
        let resp = self.transport(req).await?;
        if let Some(key) = key
            && resp.status().is_success()
        {
            let (cfg, body) = (self.config.clone(), resp.body().clone());
            let _ = tokio::task::spawn_blocking(move || cache_store(&cfg, &key, &body)).await;
        }
        execute::<E>(&url, resp)
    }

    async fn transport(&self, req: Request<()>) -> Result<Response<Bytes>> {
        let url = req.uri().to_string();
        let transport_error = |e: reqwest::Error| Error::Transport {
            url: url.clone(),
            source: Box::new(e),
        };
        let req =
            reqwest::Request::try_from(req.map(|()| Vec::<u8>::new())).map_err(transport_error)?;
        let resp = self.http.execute(req).await.map_err(transport_error)?;
        let mut out = Response::builder().status(resp.status());
        if let Some(headers) = out.headers_mut() {
            headers.clone_from(resp.headers());
        }
        let body = resp.bytes().await.map_err(transport_error)?;
        out.body(body).map_err(|e| Error::Transport {
            url,
            source: Box::new(e),
        })
    }

    crate::endpoints::endpoint_methods!(async);
}
