#![cfg(feature = "blocking")]
//! The blocking client against a local server: statuses, headers, cache.

mod common;

use std::time::Duration;

#[cfg(feature = "fs-cache")]
use common::scratch;
use common::server;
use deadlock_api::blocking::Client;
use deadlock_api::{AccountId, Error, MatchId};

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
