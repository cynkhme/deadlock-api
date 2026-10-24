//! The sans-io core: an [`Endpoint`] becomes an `http::Request`, an
//! `http::Response` becomes a typed value or an [`Error`], and the cache sits
//! between them. Transports only move bytes.

use std::time::SystemTime;

use bytes::Bytes;
use http::{HeaderMap, Request, Response, header};
use serde::de::DeserializeOwned;

use crate::error::{api_message, body_excerpt, parse_retry_after};
use crate::{Config, Error, Result};

/// One GET route of the API: its path, query, cache key and output type.
pub(crate) trait Endpoint {
    /// The decoded response.
    type Output: DeserializeOwned;

    /// The path under the server root, e.g. `/v1/matches/42/metadata`.
    fn path(&self) -> String;

    /// Query pairs in the order they are sent; lists are already comma-joined.
    fn query(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }

    /// The cache key, only for endpoints whose response never changes.
    fn cache_key(&self) -> Option<String> {
        None
    }

    /// Decodes a response or cached body.
    fn decode(body: &[u8]) -> serde_json::Result<Self::Output> {
        serde_json::from_slice(body)
    }
}

/// The full URL `e` is requested at under `cfg`.
pub(crate) fn url_for<E: Endpoint>(e: &E, cfg: &Config) -> String {
    let mut url = format!("{}{}", cfg.base_url(), e.path());
    let query = e.query();
    if !query.is_empty() {
        url.push('?');
        url.push_str(&serde_urlencoded::to_string(&query).expect("string pairs always encode"));
    }
    url
}

/// The request for `e`: GET, JSON accept, the configured user agent and, when
/// set, the API key as `X-API-KEY` (never in the URL, so it stays out of
/// caches and logs).
pub(crate) fn to_request<E: Endpoint>(e: &E, cfg: &Config) -> Request<()> {
    let mut builder = Request::get(url_for(e, cfg))
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, cfg.user_agent());
    if let Some(key) = cfg.api_key() {
        builder = builder.header("x-api-key", key);
    }
    builder
        .body(())
        .expect("the URL and header values were validated by ConfigBuilder::build")
}

/// The URL of an arbitrary GET under the server root, for `get_json` and
/// `get_raw`; query pairs are form-encoded, none means no `?`.
pub(crate) fn raw_url(cfg: &Config, path: &str, query: &[(&str, &str)]) -> String {
    let mut url = format!("{}{path}", cfg.base_url());
    if !query.is_empty() {
        url.push('?');
        url.push_str(&serde_urlencoded::to_string(query).expect("string pairs always encode"));
    }
    url
}

/// The request for an arbitrary GET, with the same headers as [`to_request`].
pub(crate) fn raw_request(cfg: &Config, url: &str) -> Request<()> {
    let mut builder = Request::get(url)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, cfg.user_agent());
    if let Some(key) = cfg.api_key() {
        builder = builder.header("x-api-key", key);
    }
    builder
        .body(())
        .expect("the header values were validated by ConfigBuilder::build")
}

/// Maps a response onto the endpoint's output or an [`Error`].
pub(crate) fn execute<E: Endpoint>(url: &str, resp: Response<Bytes>) -> Result<E::Output> {
    let body = execute_raw(url, resp)?;
    decode_at::<E>(url, &body)
}

/// Maps a response onto its 2xx body or an [`Error`].
pub(crate) fn execute_raw(url: &str, resp: Response<Bytes>) -> Result<Bytes> {
    let (parts, body) = resp.into_parts();
    if parts.status.is_success() {
        return Ok(body);
    }
    let status = parts.status.as_u16();
    let url = url.to_owned();
    Err(match status {
        429 => Error::RateLimited {
            url,
            retry_after: retry_after(&parts.headers),
        },
        503 => Error::Overloaded {
            url,
            retry_after: retry_after(&parts.headers),
        },
        404 => Error::NotFound {
            url,
            message: api_message(&body),
        },
        _ => Error::Status {
            url,
            status,
            message: api_message(&body),
            body: body_excerpt(&body),
        },
    })
}

fn retry_after(headers: &HeaderMap) -> Option<std::time::Duration> {
    let value = headers.get(header::RETRY_AFTER)?.to_str().ok()?;
    parse_retry_after(value, SystemTime::now())
}

/// The cached body for `key`, or `None` on a miss, no cache, or a read error
/// (logged as a warning).
pub(crate) fn cache_lookup(cfg: &Config, key: &str) -> Option<Vec<u8>> {
    match cfg.cache()?.get(key) {
        Ok(hit) => hit,
        Err(e) => {
            log::warn!("cache read for {key} failed, fetching instead: {e}");
            None
        }
    }
}

/// Stores a 2xx body under `key` when it is at least well-formed JSON, so a
/// one-off bad body never becomes a permanent decode error. A write error is
/// logged, not returned: the response is worth more than the cache entry.
pub(crate) fn cache_store(cfg: &Config, key: &str, body: &[u8]) {
    let Some(cache) = cfg.cache() else {
        return;
    };
    if !is_json(body) {
        return;
    }
    if let Err(e) = cache.put(key, body) {
        log::warn!("cache write for {key} failed, the next run will fetch again: {e}");
    }
}

/// Decodes a body; `location` is the URL for a response and the cache key for
/// a cached body, and names the source in the error.
pub(crate) fn decode_at<E: Endpoint>(location: &str, body: &[u8]) -> Result<E::Output> {
    E::decode(body).map_err(|source| Error::Decode {
        location: location.to_owned(),
        source,
    })
}

/// Whether a body is at least well-formed JSON, without building a tree.
fn is_json(body: &[u8]) -> bool {
    serde_json::from_slice::<serde::de::IgnoredAny>(body).is_ok()
}

/// The synchronous request flow: cache lookup, transport on a miss, cache
/// store after a 2xx, then decode. Both clients follow this sequence; the
/// async one re-states it around `await`.
///
/// A cached body that is not even JSON was never a server response (a
/// hand-edited or foreign file), so it is treated as a miss and replaced by
/// the fresh body; a cached body that is JSON but not the model stays, and
/// decoding it fails with the key as the location.
pub(crate) fn run<E, F>(e: &E, cfg: &Config, transport: F) -> Result<E::Output>
where
    E: Endpoint,
    F: FnOnce(Request<()>) -> Result<Response<Bytes>>,
{
    let key = e.cache_key();
    if let Some(key) = &key {
        if let Some(body) = cache_lookup(cfg, key) {
            if is_json(&body) {
                return decode_at::<E>(key, &body);
            }
            log::warn!("cached body for {key} is not JSON, fetching instead");
        }
    }
    let req = to_request(e, cfg);
    let url = req.uri().to_string();
    let resp = transport(req)?;
    if let Some(key) = &key {
        if resp.status().is_success() {
            cache_store(cfg, key, resp.body());
        }
    }
    execute::<E>(&url, resp)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use bytes::Bytes;
    use http::{Response, StatusCode};

    use super::*;
    use crate::{Cache, ConfigBuilder, Error};

    struct Ep;
    impl Endpoint for Ep {
        type Output = serde_json::Value;
        fn path(&self) -> String {
            "/v1/x".into()
        }
        fn query(&self) -> Vec<(&'static str, String)> {
            vec![("a", "1".into()), ("ids", "1,2".into())]
        }
        fn cache_key(&self) -> Option<String> {
            Some("x.json".into())
        }
    }

    #[derive(Default)]
    struct MemCache(Mutex<HashMap<String, Vec<u8>>>);
    impl Cache for MemCache {
        fn get(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }
        fn put(&self, key: &str, body: &[u8]) -> std::io::Result<()> {
            self.0.lock().unwrap().insert(key.into(), body.to_vec());
            Ok(())
        }
    }

    struct FailingCache;
    impl Cache for FailingCache {
        fn get(&self, _: &str) -> std::io::Result<Option<Vec<u8>>> {
            Err(std::io::Error::other("disk"))
        }
        fn put(&self, _: &str, _: &[u8]) -> std::io::Result<()> {
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        }
    }

    fn resp(status: u16, headers: &[(&str, &str)], body: &str) -> Response<Bytes> {
        let mut b = Response::builder().status(StatusCode::from_u16(status).unwrap());
        for (k, v) in headers {
            b = b.header(*k, *v);
        }
        b.body(Bytes::copy_from_slice(body.as_bytes())).unwrap()
    }

    #[test]
    fn request_url_headers_and_query() {
        let cfg = ConfigBuilder::default()
            .api_key(Some("secret".into()))
            .build()
            .unwrap();
        let req = to_request(&Ep, &cfg);
        assert_eq!(req.method(), http::Method::GET);
        assert_eq!(
            req.uri().to_string(),
            "https://api.deadlock-api.com/v1/x?a=1&ids=1%2C2"
        );
        assert_eq!(req.headers()["x-api-key"], "secret");
        assert_eq!(req.headers()["accept"], "application/json");
        assert!(
            req.headers()["user-agent"]
                .to_str()
                .unwrap()
                .starts_with("deadlock-api-rs/")
        );

        let req = to_request(&Ep, &ConfigBuilder::default().build().unwrap());
        assert!(req.headers().get("x-api-key").is_none());
    }

    #[test]
    fn base_url_with_trailing_slash_joins_cleanly() {
        let cfg = ConfigBuilder::default()
            .base_url("https://h/")
            .build()
            .unwrap();
        assert_eq!(
            to_request(&Ep, &cfg).uri().to_string(),
            "https://h/v1/x?a=1&ids=1%2C2"
        );
    }

    #[test]
    fn ok_body_is_decoded() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let v = run(&Ep, &cfg, |_| Ok(resp(200, &[], r#"{"k":1}"#))).unwrap();
        assert_eq!(v["k"], 1);
    }

    #[test]
    fn cache_hit_skips_transport() {
        let cache = MemCache::default();
        cache.put("x.json", br#"{"cached":true}"#).unwrap();
        let cfg = ConfigBuilder::default().cache(cache).build().unwrap();
        let v = run(&Ep, &cfg, |_| -> crate::Result<Response<Bytes>> {
            panic!("transport must not be called on a cache hit")
        })
        .unwrap();
        assert_eq!(v["cached"], true);
    }

    #[test]
    fn miss_stores_the_body_then_decodes() {
        let cfg = ConfigBuilder::default()
            .cache(MemCache::default())
            .build()
            .unwrap();
        run(&Ep, &cfg, |_| Ok(resp(200, &[], r#"{"k":2}"#))).unwrap();
        assert_eq!(
            cfg.cache().unwrap().get("x.json").unwrap().as_deref(),
            Some(&br#"{"k":2}"#[..])
        );
    }

    #[test]
    fn non_json_200_is_not_cached_and_is_decode_error() {
        let cfg = ConfigBuilder::default()
            .cache(MemCache::default())
            .build()
            .unwrap();
        let err = run(&Ep, &cfg, |_| Ok(resp(200, &[], "<html>"))).unwrap_err();
        assert!(
            matches!(err, Error::Decode { ref location, .. } if location.contains("/v1/x")),
            "{err:?}"
        );
        assert_eq!(cfg.cache().unwrap().get("x.json").unwrap(), None);
    }

    struct EpU8;
    impl Endpoint for EpU8 {
        type Output = u8;
        fn path(&self) -> String {
            "/v1/x".into()
        }
        fn cache_key(&self) -> Option<String> {
            Some("x.json".into())
        }
    }

    #[test]
    fn cached_json_that_is_not_the_model_is_decode_error_with_key() {
        let cache = MemCache::default();
        cache.put("x.json", br#"{"k":1}"#).unwrap();
        let cfg = ConfigBuilder::default().cache(cache).build().unwrap();
        let err = run(&EpU8, &cfg, |_| -> crate::Result<Response<Bytes>> {
            panic!("cache hit")
        })
        .unwrap_err();
        assert!(
            matches!(err, Error::Decode { ref location, .. } if location == "x.json"),
            "{err:?}"
        );
    }

    #[test]
    fn cached_non_json_garbage_falls_through_to_a_fetch() {
        let cache = MemCache::default();
        cache.put("x.json", b"nope").unwrap();
        let cfg = ConfigBuilder::default().cache(cache).build().unwrap();
        let v = run(&Ep, &cfg, |_| Ok(resp(200, &[], r#"{"fresh":true}"#))).unwrap();
        assert_eq!(v["fresh"], true);
        assert_eq!(
            cfg.cache().unwrap().get("x.json").unwrap().as_deref(),
            Some(&br#"{"fresh":true}"#[..])
        );
    }

    #[test]
    fn rate_limited_carries_retry_after() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let err = run(&Ep, &cfg, |_| Ok(resp(429, &[("retry-after", "7")], ""))).unwrap_err();
        assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(7)));
        assert_eq!(err.status(), Some(429));
    }

    #[test]
    fn overloaded_503_without_header() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let err = run(&Ep, &cfg, |_| Ok(resp(503, &[], "busy"))).unwrap_err();
        assert!(
            matches!(
                err,
                Error::Overloaded {
                    retry_after: None,
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn not_found_carries_message() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let err = run(&Ep, &cfg, |_| {
            Ok(resp(404, &[], r#"{"status":404,"error":"nope"}"#))
        })
        .unwrap_err();
        assert!(err.is_not_found());
        assert_eq!(err.message(), Some("nope"));
    }

    #[test]
    fn other_status_keeps_excerpt_and_message() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let err = run(&Ep, &cfg, |_| {
            Ok(resp(
                400,
                &[],
                r#"{"status":400,"error":"Match is not live"}"#,
            ))
        })
        .unwrap_err();
        match err {
            Error::Status {
                status,
                message,
                body,
                ..
            } => {
                assert_eq!(status, 400);
                assert_eq!(message.as_deref(), Some("Match is not live"));
                assert!(body.contains("not live"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn error_statuses_are_not_cached() {
        let cfg = ConfigBuilder::default()
            .cache(MemCache::default())
            .build()
            .unwrap();
        let _ = run(&Ep, &cfg, |_| Ok(resp(404, &[], "{}")));
        assert_eq!(cfg.cache().unwrap().get("x.json").unwrap(), None);
    }

    #[test]
    fn cache_failures_are_swallowed() {
        let cfg = ConfigBuilder::default()
            .cache(FailingCache)
            .build()
            .unwrap();
        let v = run(&Ep, &cfg, |_| Ok(resp(200, &[], r#"{"k":3}"#))).unwrap();
        assert_eq!(v["k"], 3);
    }

    #[test]
    fn transport_error_passes_through() {
        let cfg = ConfigBuilder::default().build().unwrap();
        let err = run(&Ep, &cfg, |req| {
            Err(Error::Transport {
                url: req.uri().to_string(),
                source: "timeout".into(),
            })
        })
        .unwrap_err();
        assert!(matches!(err, Error::Transport { .. }));
        assert_eq!(err.status(), None);
    }
}
