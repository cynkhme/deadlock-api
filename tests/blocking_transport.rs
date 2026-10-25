#![cfg(feature = "blocking")]
//! The blocking client against a local server: statuses, headers, cache.

mod common;

use std::time::{Duration, Instant};

#[cfg(feature = "fs-cache")]
use common::scratch;
use common::{Scripted, server};
use deadlock_api::blocking::Client;
use deadlock_api::{AccountId, AssetsQuery, Error, HeroesQuery, ItemKind, MatchId, RetryPolicy};

#[cfg(feature = "fs-cache")]
const METADATA: &str = r#"{"match_info":{"match_id":42,"duration_s":600,"players":[{"account_id":7,"player_slot":1}]}}"#;
const JSON: &[(&str, &str)] = &[("Content-Type", "application/json")];

fn client(url: &str) -> Client {
    Client::builder().base_url(url).build().unwrap()
}

#[cfg(feature = "fs-cache")]
#[test]
fn metadata_is_fetched_once_then_served_from_cache() {
    let (url, requests) = server(vec![("200 OK", JSON, METADATA)]);
    let dir = scratch("blocking-cache");
    let api = Client::builder()
        .base_url(&url)
        .cache(deadlock_api::FileCache::new(&dir))
        .build()
        .unwrap();

    let first = api.match_metadata(MatchId(42)).unwrap();
    let again = api.match_metadata(MatchId(42)).unwrap();

    assert_eq!(first.match_info.match_id, MatchId(42));
    assert_eq!(again.match_info.players[0].account_id, AccountId(7));
    assert!(dir.join("matches/42/metadata.json").is_file());
    assert_eq!(requests.lock().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "fs-cache")]
#[test]
fn non_json_200_is_not_cached() {
    let (url, _) = server(vec![("200 OK", &[], "<html>")]);
    let dir = scratch("blocking-nojson");
    let api = Client::builder()
        .base_url(&url)
        .cache(deadlock_api::FileCache::new(&dir))
        .build()
        .unwrap();
    let err = api.match_metadata(MatchId(42)).unwrap_err();
    assert!(matches!(err, Error::Decode { .. }), "{err:?}");
    assert!(!dir.join("matches/42/metadata.json").exists());
}

#[cfg(feature = "fs-cache")]
#[test]
fn cache_put_failure_returns_ok() {
    let (url, _) = server(vec![("200 OK", JSON, METADATA)]);
    let file = scratch("blocking-notadir");
    std::fs::write(&file, b"i am a file").unwrap();
    let api = Client::builder()
        .base_url(&url)
        .cache(deadlock_api::FileCache::new(&file))
        .build()
        .unwrap();
    assert_eq!(
        api.match_metadata(MatchId(42)).unwrap().match_info.match_id,
        MatchId(42)
    );
    let _ = std::fs::remove_file(&file);
}

#[test]
fn not_found_404() {
    let (url, _) = server(vec![(
        "404 Not Found",
        JSON,
        r#"{"status":404,"error":"no such match"}"#,
    )]);
    let err = client(&url).match_metadata(MatchId(1)).unwrap_err();
    assert!(err.is_not_found());
    assert_eq!(err.message(), Some("no such match"));
    assert_eq!(err.status(), Some(404));
}

#[test]
fn bad_request_400_carries_message() {
    let (url, _) = server(vec![(
        "400 Bad Request",
        JSON,
        r#"{"status":400,"error":"bad"}"#,
    )]);
    let err = client(&url).match_salts(MatchId(1)).unwrap_err();
    assert!(matches!(err, Error::Status { status: 400, .. }), "{err:?}");
    assert_eq!(err.message(), Some("bad"));
}

#[test]
fn spectate_url_400_is_none() {
    let (url, _) = server(vec![(
        "400 Bad Request",
        JSON,
        r#"{"status":400,"error":"Match 1 cannot be live"}"#,
    )]);
    assert_eq!(client(&url).match_spectate_url(MatchId(1)).unwrap(), None);
}

#[test]
fn spectate_url_200_is_some() {
    let body = r#"{"match_id":1,"broadcast_url":"https://b/1","lobby_id":5}"#;
    let (url, _) = server(vec![("200 OK", JSON, body)]);
    let got = client(&url)
        .match_spectate_url(MatchId(1))
        .unwrap()
        .unwrap();
    assert_eq!(got.broadcast_url, "https://b/1");
}

#[test]
fn rate_limited_429_with_retry_after_7() {
    let (url, _) = server(vec![("429 Too Many Requests", &[("Retry-After", "7")], "")]);
    let err = client(&url).player_rank(AccountId(1)).unwrap_err();
    assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
    assert_eq!(err.status(), Some(429));
}

#[test]
fn overloaded_503_with_http_date() {
    let (url, _) = server(vec![(
        "503 Service Unavailable",
        &[("Retry-After", "Wed, 21 Oct 2015 07:28:00 GMT")],
        "busy",
    )]);
    let err = client(&url).info().unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::ZERO));
}

#[test]
fn api_key_goes_to_header_not_url() {
    let (url, requests) = server(vec![("200 OK", JSON, r#"{"services":{}}"#)]);
    let api = Client::builder()
        .base_url(&url)
        .api_key(Some("secret".into()))
        .user_agent("test-agent/1")
        .build()
        .unwrap();
    api.health().unwrap();
    let seen = requests.lock().unwrap();
    assert_eq!(seen[0].line, "GET /v1/info/health HTTP/1.1");
    assert_eq!(seen[0].header("x-api-key"), Some("secret"));
    assert_eq!(seen[0].header("user-agent"), Some("test-agent/1"));
    assert_eq!(seen[0].header("accept"), Some("application/json"));
}

#[test]
fn connect_timeout_is_transport() {
    let api = Client::builder()
        .base_url("http://10.255.255.1:9")
        .connect_timeout(Duration::from_millis(200))
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();
    let err = api.health().unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
    assert_eq!(err.status(), None);
}

#[test]
fn get_json_and_get_raw_hit_the_path() {
    let (url, requests) = server(vec![("200 OK", JSON, r#"{"k":[1,2]}"#)]);
    let api = client(&url);
    let v: serde_json::Value = api
        .get_json("/v1/analytics/x", &[("a", "1"), ("b", "two words")])
        .unwrap();
    assert_eq!(v["k"][1], 2);
    let raw = api.get_raw("/v1/analytics/x", &[]).unwrap();
    assert_eq!(&raw[..], br#"{"k":[1,2]}"#);
    let seen = requests.lock().unwrap();
    assert_eq!(seen[0].line, "GET /v1/analytics/x?a=1&b=two+words HTTP/1.1");
    assert_eq!(seen[1].line, "GET /v1/analytics/x HTTP/1.1");
}

#[test]
fn empty_id_list_makes_no_request() {
    let (url, requests) = server(vec![(
        "500 Internal Server Error",
        &[],
        "must not be called",
    )]);
    let api = client(&url);
    assert!(api.steam_profiles(&[]).unwrap().is_empty());
    assert!(api.player_ranks(&[]).unwrap().is_empty());
    assert!(
        api.hero_stats(&[], &deadlock_api::HeroStatsQuery::default())
            .unwrap()
            .is_empty()
    );
    assert!(requests.lock().unwrap().is_empty());
}

#[test]
fn builder_rejects_bad_config() {
    let err = Client::builder().base_url("not a url").build().unwrap_err();
    assert!(matches!(err, Error::Config { .. }), "{err:?}");
}

#[test]
fn spectate_url_400_for_other_reasons_is_an_error() {
    let (url, _) = server(vec![(
        "400 Bad Request",
        JSON,
        r#"{"status":400,"error":"invalid match id"}"#,
    )]);
    let err = client(&url).match_spectate_url(MatchId(1)).unwrap_err();
    assert!(matches!(err, Error::Status { status: 400, .. }), "{err:?}");
}

#[test]
fn spectate_url_200_schema_shape_decodes() {
    let (url, _) = server(vec![(
        "200 OK",
        JSON,
        r#"{"broadcast_url":"https://b/1","lobby_id":1}"#,
    )]);
    let got = client(&url)
        .match_spectate_url(MatchId(1))
        .unwrap()
        .unwrap();
    assert_eq!(
        (got.broadcast_url.as_str(), got.lobby_id),
        ("https://b/1", Some(1))
    );
}

#[test]
fn redirects_are_not_followed() {
    let (url, requests) = server(vec![
        ("302 Found", &[("Location", "/v1/info")], ""),
        ("200 OK", JSON, r#"{"services":{}}"#),
    ]);
    let err = client(&url).health().unwrap_err();
    assert_eq!(err.status(), Some(302), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn api_key_query_pair_is_rejected() {
    let (url, requests) = server(vec![("200 OK", JSON, "{}")]);
    let api = client(&url);
    let err = api.get_raw("/v1/x", &[("api_key", "leak")]).unwrap_err();
    assert!(matches!(err, Error::Config { .. }), "{err:?}");
    assert!(requests.lock().unwrap().is_empty());
}

#[test]
fn invalid_path_is_an_error_not_a_panic() {
    let (url, _) = server(vec![("200 OK", JSON, "{}")]);
    let api = client(&url);
    let err = api.get_raw("/v1/a b", &[]).unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
    let err = api
        .leaderboard(deadlock_api::Region::Unknown("a b".into()))
        .unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
}

#[test]
fn raw_path_cannot_move_the_request_off_the_base_host() {
    let (url, requests) = server(vec![("200 OK", JSON, "{}")]);
    let api = client(&url);
    for path in [
        "@evil.example/x",
        ".evil.example/x",
        "v1/info",
        "/v1/x?api_key=k",
        "/v1/x#f",
    ] {
        let err = api.get_raw(path, &[]).unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest { .. }),
            "{path}: {err:?}"
        );
        assert!(!err.to_string().contains("api_key"), "{err}");
    }
    let err = api
        .leaderboard(deadlock_api::Region::Unknown("Europe?x=1".into()))
        .unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
    assert!(requests.lock().unwrap().is_empty());
}

#[cfg(feature = "gzip")]
#[test]
fn gzip_is_asked_for_and_decoded() {
    let (url, requests) = common::server_bytes(vec![(
        "200 OK",
        &[
            ("Content-Type", "application/json"),
            ("Content-Encoding", "gzip"),
        ],
        common::HEALTH_GZIP.to_vec(),
    )]);
    let health = client(&url).health().unwrap();
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

#[test]
fn retry_is_off_by_default() {
    let (url, requests) = server(vec![BUSY, HEALTHY]);
    let err = client(&url).health().unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn overloaded_503_is_retried_until_it_succeeds() {
    let (url, requests) = server(vec![BUSY, BUSY, HEALTHY]);
    retrying(&url, quick(3)).health().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 3);
}

#[test]
fn retries_stop_after_max_attempts() {
    let (url, requests) = server(vec![BUSY, BUSY, HEALTHY]);
    let err = retrying(&url, quick(2)).health().unwrap_err();
    assert!(matches!(err, Error::Overloaded { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn not_found_is_not_retried() {
    let (url, requests) = server(vec![("404 Not Found", &[], ""), HEALTHY]);
    let err = retrying(&url, quick(3)).health().unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn rate_limit_without_retry_after_is_not_retried() {
    let (url, requests) = server(vec![("429 Too Many Requests", &[], ""), HEALTHY]);
    let err = retrying(&url, quick(3)).health().unwrap_err();
    assert!(matches!(err, Error::RateLimited { .. }), "{err:?}");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn rate_limit_beyond_max_wait_returns_at_once() {
    let (url, requests) = server(vec![
        ("429 Too Many Requests", &[("Retry-After", "120")], ""),
        HEALTHY,
    ]);
    let start = Instant::now();
    let err = retrying(&url, quick(3)).health().unwrap_err();
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(err.retry_after(), Some(Duration::from_secs(120)));
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn a_zero_or_past_retry_after_still_pauses_for_the_backoff() {
    let (url, requests) = server(vec![
        (
            "429 Too Many Requests",
            &[("Retry-After", "Wed, 21 Oct 2015 07:28:00 GMT")],
            "",
        ),
        ("429 Too Many Requests", &[("Retry-After", "0")], ""),
        HEALTHY,
    ]);
    let policy = RetryPolicy::new(3).base_delay(Duration::from_millis(200));
    let start = Instant::now();
    retrying(&url, policy).health().unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(600),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(requests.lock().unwrap().len(), 3);
}

#[test]
fn a_longer_retry_after_is_waited_out() {
    let (url, requests) = server(vec![
        ("503 Service Unavailable", &[("Retry-After", "1")], ""),
        HEALTHY,
    ]);
    let start = Instant::now();
    retrying(&url, quick(2)).health().unwrap();
    assert!(
        start.elapsed() >= Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn get_raw_is_retried() {
    let (url, requests) = server(vec![BUSY, ("200 OK", &[], "x")]);
    let body = retrying(&url, quick(2)).get_raw("/v1/x", &[]).unwrap();
    assert_eq!(&body[..], b"x");
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn a_refused_connection_is_retried() {
    let policy = RetryPolicy::new(3).base_delay(Duration::from_millis(100));
    let start = Instant::now();
    let err = retrying(&common::refused_url(), policy)
        .health()
        .unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
    assert!(
        start.elapsed() >= Duration::from_millis(300),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn a_timeout_after_sending_is_not_retried() {
    let (url, requests) = common::silent_server();
    let api = Client::builder()
        .base_url(&url)
        .timeout(Duration::from_millis(300))
        .retry(quick(3))
        .build()
        .unwrap();
    let err = api.health().unwrap_err();
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

#[test]
fn custom_transport_gets_the_built_request_and_its_response_is_decoded() {
    let mock = common::Mock::new(vec![(200, JSON, r#"{"services":{"redis":true}}"#)]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .api_key(Some("secret".into()))
        .user_agent("test-agent/1")
        .transport(mock)
        .build()
        .unwrap();
    let health = api.health().unwrap();
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

#[test]
fn custom_transport_error_is_transport_and_never_retried() {
    let mock = common::Mock::new(vec![]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(mock)
        .retry(quick(3))
        .build()
        .unwrap();
    let err = api.health().unwrap_err();
    let Error::Transport { source, .. } = &err else {
        panic!("{err:?}");
    };
    assert_eq!(source.to_string(), "mock transport failure");
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[test]
fn custom_transport_statuses_are_mapped_and_retried() {
    let mock = common::Mock::new(vec![(503, &[], "busy"), (200, JSON, r#"{"services":{}}"#)]);
    let seen = mock.seen.clone();
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(mock)
        .retry(quick(2))
        .build()
        .unwrap();
    api.health().unwrap();
    assert_eq!(seen.lock().unwrap().len(), 2);
    let err = mocked(common::Mock::new(vec![(404, &[], "")]))
        .health()
        .unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
}

#[test]
fn custom_transport_keeps_the_short_cuts() {
    let mock = common::Mock::new(vec![(
        400,
        JSON,
        r#"{"status":400,"error":"Match 1 cannot be live"}"#,
    )]);
    let seen = mock.seen.clone();
    let api = mocked(mock);
    assert!(api.steam_profiles(&[]).unwrap().is_empty());
    assert_eq!(seen.lock().unwrap().len(), 0);
    assert_eq!(api.match_spectate_url(MatchId(1)).unwrap(), None);
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[cfg(feature = "fs-cache")]
#[test]
fn custom_transport_responses_are_cached() {
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
        let meta = api.match_metadata(MatchId(42)).unwrap();
        assert_eq!(meta.match_info.match_id, MatchId(42));
    }
    assert_eq!(seen.lock().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn asset_methods_encode_the_path_and_decode_the_recorded_bodies() {
    let mock = common::Mock::new(vec![
        (200, JSON, include_str!("fixtures/assets_heroes.json")),
        (404, &[], ""),
        (200, JSON, include_str!("fixtures/assets_items.json")),
    ]);
    let seen = mock.seen.clone();
    let api = mocked(mock);
    let active = HeroesQuery {
        only_active: Some(true),
        ..HeroesQuery::default()
    };
    assert_eq!(api.heroes(&active).unwrap().len(), 3);
    let q = AssetsQuery::default();
    let err = api.hero_by_name("Mo & Krill", &q).unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
    let items = api.items_by_kind(ItemKind::Weapon, &q).unwrap();
    assert_eq!(items.len(), 7);
    let err = api.hero_by_name("a/b", &q).unwrap_err();
    assert!(matches!(err, Error::InvalidRequest { .. }), "{err:?}");
    let uris: Vec<String> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|r| r.uri().to_string())
        .collect();
    let base = common::NOWHERE;
    assert_eq!(
        uris,
        [
            format!("{base}/v1/assets/heroes?only_active=true"),
            format!("{base}/v1/assets/heroes/by-name/Mo%20%26%20Krill"),
            format!("{base}/v1/assets/items/by-type/weapon"),
        ]
    );
}

#[test]
fn a_body_over_64_mib_is_refused_and_one_over_10_mib_is_read() {
    const MIB: usize = 1024 * 1024;
    let (url, _) = common::server_bytes(vec![
        ("200 OK", &[], vec![b'x'; 11 * MIB]),
        ("200 OK", &[], vec![b'x'; 64 * MIB + 1]),
    ]);
    let api = Client::builder().base_url(&url).build().unwrap();
    assert_eq!(api.get_raw("/v1/x", &[]).unwrap().len(), 11 * MIB);
    let err = api.get_raw("/v1/x", &[]).unwrap_err();
    assert!(matches!(err, Error::Transport { .. }), "{err:?}");
}

#[test]
fn a_shared_or_boxed_transport_is_a_transport() {
    let shared = std::sync::Arc::new(common::Mock::new(vec![(200, JSON, r#"{"services":{}}"#)]));
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(shared.clone())
        .build()
        .unwrap();
    api.health().unwrap();
    assert_eq!(shared.seen.lock().unwrap().len(), 1);
    let boxed: Box<dyn deadlock_api::blocking::Transport> =
        Box::new(common::Mock::new(vec![(404, &[], "")]));
    let api = Client::builder()
        .base_url(common::NOWHERE)
        .transport(boxed)
        .build()
        .unwrap();
    assert!(api.health().unwrap_err().is_not_found());
}
