//! The asynchronous client, on [reqwest](https://docs.rs/reqwest) and tokio.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response};
use serde::de::DeserializeOwned;

use crate::endpoint::{
    Endpoint, cache_lookup, cache_store, decode_at, execute, execute_raw, is_json, raw_request,
    raw_url, to_request,
};
use crate::retry::copy_request;
use crate::{
    AccountId, AccountRank, ActiveMatch, ActiveMatchesQuery, ApiInfo, BoxError, Config,
    ConfigBuilder, EnemyStats, EnemyStatsQuery, Error, Health, HeroId, HeroStats, HeroStatsQuery,
    Leaderboard, MatchFilter, MatchHistoryEntry, MatchId, MatchMetadata, MatchSalts, MateStats,
    MateStatsQuery, RankDistributionEntry, RankResponse, RecentMatch, Region, Result, RetryPolicy,
    SpectateUrl, SteamProfile, SteamSearchQuery,
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
#[derive(Clone)]
pub struct Client {
    transport: Arc<dyn Transport>,
    unsent: fn(&Error) -> bool,
    config: Arc<Config>,
    retry: Option<RetryPolicy>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .field("retry", &self.retry)
            .finish_non_exhaustive()
    }
}

/// The future a [`Transport`] returns.
pub type TransportFuture<'a> =
    Pin<Box<dyn Future<Output = std::result::Result<Response<Bytes>, BoxError>> + Send + 'a>>;

/// Sends a request and returns the response with its whole body: the
/// seam between the async [`Client`] and the HTTP library under it.
///
/// The client builds every request (URL, `X-API-KEY`, `Accept`,
/// `User-Agent`) and keeps everything around the exchange: the cache, the
/// retry policy, the mapping of statuses onto [`Error`] and the decoding. A
/// transport set with [`ClientBuilder::transport`] replaces reqwest, so the
/// timeouts are its own business, and it should not follow redirects, or the
/// API key goes along to the other host. Its error becomes
/// [`Error::Transport`] and is never retried: the client cannot tell whether
/// the request reached the server. The client still needs a tokio runtime:
/// it calls the cache on tokio's blocking pool and sleeps between retries
/// on tokio's timer.
///
/// A transport that serves recorded responses runs a consumer's whole flow,
/// cache and decoding included, without a network.
///
/// # Examples
///
/// ```
/// use bytes::Bytes;
/// use deadlock_api::{BoxError, Client, Transport, TransportFuture};
///
/// struct Recorded;
///
/// impl Transport for Recorded {
///     fn send(&self, request: http::Request<()>) -> TransportFuture<'_> {
///         Box::pin(async move {
///             assert_eq!(request.uri().path(), "/v1/info/health");
///             let body = r#"{"services":{"clickhouse":true}}"#;
///             Ok::<_, BoxError>(http::Response::new(Bytes::from(body)))
///         })
///     }
/// }
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> deadlock_api::Result<()> {
/// let api = Client::builder().transport(Recorded).build()?;
/// assert_eq!(api.health().await?.services.get("clickhouse"), Some(&true));
/// # Ok(()) }
/// ```
pub trait Transport: Send + Sync {
    /// Sends `request` and reads the whole response body.
    ///
    /// # Errors
    ///
    /// Any failure to produce a response; a non-2xx status is a response,
    /// not an error.
    fn send(&self, request: Request<()>) -> TransportFuture<'_>;
}

/// The built-in transport.
struct Reqwest(reqwest::Client);

impl Transport for Reqwest {
    fn send(&self, request: Request<()>) -> TransportFuture<'_> {
        Box::pin(async move {
            let request = reqwest::Request::try_from(request.map(|()| Vec::<u8>::new()))?;
            let resp = self.0.execute(request).await?;
            let mut out = Response::builder().status(resp.status());
            if let Some(headers) = out.headers_mut() {
                headers.clone_from(resp.headers());
            }
            let body = resp.bytes().await?;
            Ok(out.body(body)?)
        })
    }
}

/// Builder for [`Client`]; what it sets, apart from the retry policy and the
/// transport, is readable afterwards through [`Client::config`].
#[derive(Default)]
pub struct ClientBuilder {
    config: ConfigBuilder,
    retry: Option<RetryPolicy>,
    transport: Option<Arc<dyn Transport>>,
}

impl std::fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("config", &self.config)
            .field("retry", &self.retry)
            .field("custom_transport", &self.transport.is_some())
            .finish()
    }
}

builder_setters!(ClientBuilder);

impl ClientBuilder {
    /// Sends every request through `transport` instead of reqwest; see
    /// [`Transport`] for what it takes over. The connection timeout and the
    /// whole-request timeout then do not apply.
    pub fn transport(mut self, transport: impl Transport + 'static) -> Self {
        self.transport = Some(Arc::new(transport));
        self
    }

    /// Validates the settings and builds the client.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] when the base URL has no scheme or host, or the user
    /// agent or API key is not a valid header value; [`Error::Transport`] when
    /// reqwest cannot initialise its TLS backend (never with a custom
    /// transport).
    pub fn build(self) -> Result<Client> {
        let config = self.config.build()?;
        let unsent: fn(&Error) -> bool = match self.transport {
            Some(_) => |_| false,
            None => reqwest_unsent,
        };
        let transport = match self.transport {
            Some(custom) => custom,
            None => Arc::new(Reqwest(reqwest_client(&config)?)),
        };
        Ok(Client {
            transport,
            unsent,
            config: Arc::new(config),
            retry: self.retry,
        })
    }
}

/// The reqwest client of a [`Client`] without a custom transport.
fn reqwest_client(config: &Config) -> Result<reqwest::Client> {
    let builder = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(config.user_agent())
        .connect_timeout(config.connect_timeout())
        .timeout(config.timeout());
    #[cfg(feature = "rustls")]
    let builder = builder.use_rustls_tls();
    #[cfg(all(feature = "native-tls", not(feature = "rustls")))]
    let builder = builder.use_native_tls();
    builder.build().map_err(|e| Error::Transport {
        url: config.base_url().to_owned(),
        source: Box::new(e),
    })
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
        let Some(policy) = &self.retry else {
            return self.send_once(req).await;
        };
        let mut attempts = 1;
        loop {
            let result = self.send_once(copy_request(&req)).await;
            let Some(pause) = policy.pause(attempts, &result, self.unsent) else {
                return result;
            };
            log::info!("retrying {} in {pause:?}", req.uri());
            tokio::time::sleep(pause).await;
            attempts += 1;
        }
    }

    async fn send_once(&self, req: Request<()>) -> Result<Response<Bytes>> {
        let url = req.uri().to_string();
        self.transport
            .send(req)
            .await
            .map_err(|source| Error::Transport { url, source })
    }

    crate::endpoints::endpoint_methods!(async);
}

/// Whether reqwest failed before the request reached the server.
fn reqwest_unsent(error: &Error) -> bool {
    matches!(error, Error::Transport { source, .. }
        if source.downcast_ref::<reqwest::Error>().is_some_and(reqwest::Error::is_connect))
}
