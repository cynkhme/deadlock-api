#![cfg(feature = "async")]
//! The async client against a local server: the same scenarios as the
//! blocking client, around `await`.

mod common;

use std::time::{Duration, Instant};

#[cfg(feature = "fs-cache")]
use common::scratch;
use common::{Scripted, server};
use deadlock_api::{AccountId, Client, Error, MatchId, RetryPolicy};

#[cfg(feature = "fs-cache")]
const METADATA: &str = r#"{"match_info":{"match_id":42,"duration_s":600,"players":[{"account_id":7,"player_slot":1}]}}"#;
const JSON: &[(&str, &str)] = &[("Content-Type", "application/json")];

fn client(url: &str) -> Client {
    Client::builder().base_url(url).build().unwrap()
}

#[cfg(feature = "fs-cache")]
#[tokio::test]
async fn metadata_is_fetched_once_then_served_from_cache() {
    let (url, requests) = server(vec![("200 OK", JSON, METADATA)]);
    let dir = scratch("async-cache");
    let api = Client::builder()
        .base_url(&url)
        .cache(deadlock_api::FileCache::new(&dir))
        .build()
        .unwrap();

    let first = api.match_metadata(MatchId(42)).await.unwrap();
    let again = api.match_metadata(MatchId(42)).await.unwrap();

    assert_eq!(first.match_info.match_id, MatchId(42));
    assert_eq!(again.match_info.players[0].account_id, AccountId(7));
    assert!(dir.join("matches/42/metadata.json").is_file());
    assert_eq!(requests.lock().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "fs-cache")]
#[tokio::test]
async fn cache_put_failure_returns_ok() {
    let (url, _) = server(vec![("200 OK", JSON, METADATA)]);
    let file = scratch("async-notadir");
    std::fs::write(&file, b"i am a file").unwrap();
    let api = Client::builder()
        .base_url(&url)
        .cache(deadlock_api::FileCache::new(&file))
        .build()
        .unwrap();
    assert_eq!(
        api.match_metadata(MatchId(42))
            .await
            .unwrap()
            .match_info
            .match_id,
        MatchId(42)
    );
    let _ = std::fs::remove_file(&file);
}

#[tokio::test]
async fn not_found_404_and_bad_request_400() {
    let (url, _) = server(vec![
        (
            "404 Not Found",
            JSON,
            r#"{"status":404,"error":"no such match"}"#,
        ),
        ("400 Bad Request", JSON, r#"{"status":400,"error":"bad"}"#),
    ]);
    let api = client(&url);
    let err = api.match_metadata(MatchId(1)).await.unwrap_err();
    assert!(err.is_not_found());
    assert_eq!(err.message(), Some("no such match"));
    let err = api.match_salts(MatchId(1)).await.unwrap_err();
    assert!(matches!(err, Error::Status { status: 400, .. }), "{err:?}");
}

#[tokio::test]
async fn spectate_url_400_is_none() {
    let (url, _) = server(vec![(
        "400 Bad Request",
        JSON,
        r#"{"status":400,"error":"Match 1 cannot be live"}"#,
    )]);
    assert_eq!(
        client(&url).match_spectate_url(MatchId(1)).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn rate_limited_and_overloaded_carry_retry_after() {
    let (url, _) = server(vec![
        ("429 Too Many Requests", &[("Retry-After", "7")], ""),
        (
            "503 Service Unavailable",
            &[("Retry-After", "Wed, 21 Oct 2015 07:28:00 GMT")],
            "busy",
        ),
    ]);
    let api = client(&url);
    let err = api.player_rank(AccountId(1)).await.unwrap_err();
    assert_eq!(
        (err.status(), err.retry_after()),
        (Some(429), Some(Duration::from_secs(7)))
    );
    let err = api.info().await.unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::ZERO));
}

#[tokio::test]
async fn api_key_goes_to_header_not_url() {
    let (url, requests) = server(vec![("200 OK", JSON, r#"{"services":{}}"#)]);
    let api = Client::builder()
        .base_url(&url)
        .api_key(Some("secret".into()))
        .user_agent("test-agent/1")
        .build()
        .unwrap();
    api.health().await.unwrap();
    let seen = requests.lock().unwrap();
    assert_eq!(seen[0].line, "GET /v1/info/health HTTP/1.1");
    assert_eq!(seen[0].header("x-api-key"), Some("secret"));
    assert_eq!(seen[0].header("user-agent"), Some("test-agent/1"));
    assert_eq!(seen[0].header("accept"), Some("application/json"));
}

#[tokio::test]
async fn connect_timeout_is_transport() {
    let api = Client::builder()
        .base_url("http://10.255.255.1:9")
        .connect_timeout(Duration::from_millis(200))
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();
    let err = api.health().await.unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
}

#[tokio::test]
async fn get_json_and_get_raw_hit_the_path() {
    let (url, requests) = server(vec![("200 OK", JSON, r#"{"k":[1,2]}"#)]);
    let api = client(&url);
    let v: serde_json::Value = api
        .get_json("/v1/analytics/x", &[("a", "1")])
        .await
        .unwrap();
    assert_eq!(v["k"][1], 2);
    let raw = api.get_raw("/v1/analytics/x", &[]).await.unwrap();
    assert_eq!(&raw[..], br#"{"k":[1,2]}"#);
    let seen = requests.lock().unwrap();
    assert_eq!(seen[0].line, "GET /v1/analytics/x?a=1 HTTP/1.1");
    assert_eq!(seen[1].line, "GET /v1/analytics/x HTTP/1.1");
}

#[tokio::test]
async fn empty_id_list_makes_no_request() {
    let (url, requests) = server(vec![(
        "500 Internal Server Error",
        &[],
        "must not be called",
    )]);
    let api = client(&url);
    assert!(api.steam_profiles(&[]).await.unwrap().is_empty());
    assert!(api.player_ranks(&[]).await.unwrap().is_empty());
    assert!(
        api.hero_stats(&[], &deadlock_api::HeroStatsQuery::default())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn client_is_send_and_usable_from_a_spawned_task() {
    let (url, _) = server(vec![("200 OK", JSON, r#"{"services":{}}"#)]);
    let api = client(&url);
    let handle = tokio::spawn(async move { api.health().await.map(|h| h.services.len()) });
    assert_eq!(handle.await.unwrap().unwrap(), 0);
}

#[tokio::test]
async fn spectate_url_400_for_other_reasons_is_an_error() {
    let (url, _) = server(vec![(
        "400 Bad Request",
        JSON,
        r#"{"status":400,"error":"invalid match id"}"#,
    )]);
    let err = client(&url)
        .match_spectate_url(MatchId(1))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Status { status: 400, .. }), "{err:?}");
}

#[tokio::test]
async fn redirects_are_not_followed() {
    let (url, requests) = server(vec![
        ("302 Found", &[("Location", "/v1/info")], ""),
        ("200 OK", JSON, r#"{"services":{}}"#),
    ]);
    let err = client(&url).health().await.unwrap_err();
    assert_eq!(err.status(), Some(302), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn api_key_query_pair_is_rejected() {
    let (url, _) = server(vec![("200 OK", JSON, "{}")]);
    let err = client(&url)
        .get_raw("/v1/x", &[("api_key", "leak")])
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config { .. }), "{err:?}");
}

#[tokio::test]
async fn invalid_path_is_an_error_not_a_panic() {
    let (url, _) = server(vec![("200 OK", JSON, "{}")]);
    let err = client(&url).get_raw("/v1/a b", &[]).await.unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
}

#[tokio::test]
async fn raw_path_cannot_move_the_request_off_the_base_host() {
    let (url, _) = server(vec![("200 OK", JSON, "{}")]);
    let api = client(&url);
    for path in [
        "@evil.example/x",
        ".evil.example/x",
        "v1/info",
        "/v1/x?api_key=k",
        "/v1/x#f",
    ] {
        let err = api.get_raw(path, &[]).await.unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest { .. }),
            "{path}: {err:?}"
        );
        assert!(!err.to_string().contains("api_key"), "{err}");
    }
    let err = api
        .leaderboard(deadlock_api::Region::Unknown("Europe?x=1".into()))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
}

#[cfg(feature = "gzip")]
#[tokio::test]
async fn gzip_is_asked_for_and_decoded() {
    let (url, requests) = common::server_bytes(vec![(
        "200 OK",
        &[
            ("Content-Type", "application/json"),
            ("Content-Encoding", "gzip"),
        ],
        common::HEALTH_GZIP.to_vec(),
    )]);
    let health = client(&url).health().await.unwrap();
    assert_eq!(health.services.get("redis"), Some(&true));
    let seen = requests.lock().unwrap();
    let accept = seen[0].header("accept-encoding").unwrap_or_default();
    assert!(accept.contains("gzip"), "{accept:?}");
}

fn retrying(url: &str, policy: RetryPolicy) -> Client {
    Client::builder()
        .base_url(url)
        .retry(policy)
        .build()
        .unwrap()
}

fn quick(attempts: u32) -> RetryPolicy {
    RetryPolicy::new(attempts).base_delay(Duration::from_millis(1))
}

const BUSY: Scripted = ("503 Service Unavailable", &[], "busy");
const HEALTHY: Scripted = ("200 OK", JSON, r#"{"services":{}}"#);

#[tokio::test]
async fn retry_is_off_by_default() {
    let (url, requests) = server(vec![BUSY, HEALTHY]);
    let err = client(&url).health().await.unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn overloaded_503_is_retried_until_it_succeeds() {
    let (url, requests) = server(vec![BUSY, BUSY, HEALTHY]);
    retrying(&url, quick(3)).health().await.unwrap();
    assert_eq!(requests.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn retries_stop_after_max_attempts() {
    let (url, requests) = server(vec![BUSY, BUSY, HEALTHY]);
    let err = retrying(&url, quick(2)).health().await.unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn not_found_is_not_retried() {
    let (url, requests) = server(vec![("404 Not Found", &[], ""), HEALTHY]);
    let err = retrying(&url, quick(3)).health().await.unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn rate_limit_without_retry_after_is_not_retried() {
    let (url, requests) = server(vec![("429 Too Many Requests", &[], ""), HEALTHY]);
    let err = retrying(&url, quick(3)).health().await.unwrap_err();
    assert!(matches!(err, Error::RateLimited { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn rate_limit_beyond_max_wait_returns_at_once() {
    let (url, requests) = server(vec![
        ("429 Too Many Requests", &[("Retry-After", "120")], ""),
        HEALTHY,
    ]);
    let start = Instant::now();
    let err = retrying(&url, quick(3)).health().await.unwrap_err();
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(err.retry_after(), Some(Duration::from_secs(120)));
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn zero_retry_after_still_pauses_for_the_backoff() {
    let (url, requests) = server(vec![
        ("429 Too Many Requests", &[("Retry-After", "0")], ""),
        HEALTHY,
    ]);
    let policy = RetryPolicy::new(2).base_delay(Duration::from_millis(200));
    let start = Instant::now();
    retrying(&url, policy).health().await.unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(200),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_longer_retry_after_is_waited_out() {
    let (url, requests) = server(vec![
        ("503 Service Unavailable", &[("Retry-After", "1")], ""),
        HEALTHY,
    ]);
    let start = Instant::now();
    retrying(&url, quick(2)).health().await.unwrap();
    assert!(
        start.elapsed() >= Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn get_raw_is_retried() {
    let (url, requests) = server(vec![BUSY, ("200 OK", &[], "x")]);
    let body = retrying(&url, quick(2))
        .get_raw("/v1/x", &[])
        .await
        .unwrap();
    assert_eq!(&body[..], b"x");
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_refused_connection_is_retried() {
    let policy = RetryPolicy::new(3).base_delay(Duration::from_millis(100));
    let start = Instant::now();
    let err = retrying(&common::refused_url(), policy)
        .health()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
    assert!(
        start.elapsed() >= Duration::from_millis(300),
        "{:?}",
        start.elapsed()
    );
}

#[tokio::test]
async fn a_timeout_after_sending_is_not_retried() {
    let (url, requests) = common::silent_server();
    let api = Client::builder()
        .base_url(&url)
        .timeout(Duration::from_millis(300))
        .retry(quick(3))
        .build()
        .unwrap();
    let err = api.health().await.unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

fn mocked(mock: common::Mock) -> Client {
    Client::builder()
        .base_url(common::NOWHERE)
        .transport(mock)
        .build()
        .unwrap()
}

#[tokio::test]
async fn custom_transport_gets_the_built_request_and_its_response_is_decoded() {
    let mock = common::Mock::new(vec![(200, JSON, r#"{"services":{"redis":true}}"#)]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .api_key(Some("secret".into()))
        .user_agent("test-agent/1")
        .transport(mock)
        .build()
        .unwrap();
    let health = api.health().await.unwrap();
    assert_eq!(health.services.get("redis"), Some(&true));
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].uri().to_string(),
        format!("{}/v1/info/health", common::NOWHERE)
    );
    assert_eq!(seen[0].headers()["x-api-key"], "secret");
    assert_eq!(seen[0].headers()["user-agent"], "test-agent/1");
}

#[tokio::test]
async fn custom_transport_error_is_transport_and_never_retried() {
    let mock = common::Mock::new(vec![]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(mock)
        .retry(quick(3))
        .build()
        .unwrap();
    let err = api.health().await.unwrap_err();
    let Error::Transport { source, .. } = &err else {
        panic!("{err:?}");
    };
    assert_eq!(source.to_string(), "mock transport failure");
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn custom_transport_statuses_are_mapped_and_retried() {
    let mock = common::Mock::new(vec![(503, &[], "busy"), (200, JSON, r#"{"services":{}}"#)]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(mock)
        .retry(quick(2))
        .build()
        .unwrap();
    api.health().await.unwrap();
    assert_eq!(seen.lock().unwrap().len(), 2);
    let err = mocked(common::Mock::new(vec![(404, &[], "")]))
        .health()
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
}

#[tokio::test]
async fn custom_transport_keeps_the_short_cuts() {
    let mock = common::Mock::new(vec![(
        400,
        JSON,
        r#"{"status":400,"error":"Match 1 cannot be live"}"#,
    )]);
    let seen = mock.seen.clone();
    let api = mocked(mock);
    assert!(api.steam_profiles(&[]).await.unwrap().is_empty());
    assert_eq!(seen.lock().unwrap().len(), 0);
    assert_eq!(api.match_spectate_url(MatchId(1)).await.unwrap(), None);
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[cfg(feature = "fs-cache")]
#[tokio::test]
async fn custom_transport_responses_are_cached() {
    let dir = scratch("custom-transport-cache");
    let mock = common::Mock::new(vec![(200, JSON, METADATA)]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .cache(deadlock_api::FileCache::new(&dir))
        .transport(mock)
        .build()
        .unwrap();
    for _ in 0..2 {
        let meta = api.match_metadata(MatchId(42)).await.unwrap();
        assert_eq!(meta.match_info.match_id, MatchId(42));
    }
    assert_eq!(seen.lock().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
