//! The error type and the helpers that map a server response onto it.

use std::time::{Duration, SystemTime};

/// The crate's result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything a request can fail with.
///
/// Non-2xx statuses map onto their own variants so a caller can match on the
/// outcome instead of on a number; [`Error::retry_after`] and
/// [`Error::status`] give the common questions one call each.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The server answered 429: too many requests for this key or address.
    #[error("rate limited by deadlock-api ({url})")]
    RateLimited {
        /// The request URL.
        url: String,
        /// The server's `Retry-After`, if it sent one that parsed.
        retry_after: Option<Duration>,
    },
    /// The server answered 503: it is shedding load.
    #[error("deadlock-api is overloaded ({url})")]
    Overloaded {
        /// The request URL.
        url: String,
        /// The server's `Retry-After`, if it sent one that parsed.
        retry_after: Option<Duration>,
    },
    /// The server answered 404: it has no data for this request. It does not
    /// say whether the match or player exists.
    #[error("deadlock-api has no data for it ({url})")]
    NotFound {
        /// The request URL.
        url: String,
        /// The `error` field of the server's JSON error body, if it sent one.
        message: Option<String>,
    },
    /// Any other non-2xx status.
    #[error("{url} answered {status}")]
    Status {
        /// The request URL.
        url: String,
        /// The HTTP status code.
        status: u16,
        /// The `error` field of the server's JSON error body, if it sent one.
        message: Option<String>,
        /// The first 512 bytes of the body, lossily decoded.
        body: String,
    },
    /// The request never produced a response: connection, TLS or timeout.
    #[error("request to {url} failed")]
    Transport {
        /// The request URL.
        url: String,
        /// The transport's own error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// A 2xx body or a cached body did not decode into the expected type.
    #[error("decoding {location}")]
    Decode {
        /// The request URL, or the cache key when the body came from the cache.
        location: String,
        /// The JSON error.
        #[source]
        source: serde_json::Error,
    },
}

impl Error {
    /// The server's `Retry-After` for a 429 or 503, if it sent one that parsed.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::RateLimited { retry_after, .. } | Error::Overloaded { retry_after, .. } => {
                *retry_after
            }
            _ => None,
        }
    }

    /// Whether this is a 404.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound { .. })
    }

    /// The HTTP status behind this error; `None` when no response arrived or
    /// the response did not decode.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::RateLimited { .. } => Some(429),
            Error::Overloaded { .. } => Some(503),
            Error::NotFound { .. } => Some(404),
            Error::Status { status, .. } => Some(*status),
            Error::Transport { .. } | Error::Decode { .. } => None,
        }
    }

    /// The `error` field of the server's JSON error body, when the response
    /// carried one.
    pub fn message(&self) -> Option<&str> {
        match self {
            Error::NotFound { message, .. } | Error::Status { message, .. } => message.as_deref(),
            _ => None,
        }
    }
}

/// Parses a `Retry-After` header value: whole seconds, or an HTTP-date taken
/// relative to `now` (a date in the past gives zero).
pub(crate) fn parse_retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    let value = value.trim();
    if let Ok(secs) = value.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }
    let at = httpdate::parse_http_date(value).ok()?;
    Some(at.duration_since(now).unwrap_or(Duration::ZERO))
}

/// The `error` field of the server's `{"status": …, "error": "…"}` body.
pub(crate) fn api_message(body: &[u8]) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct ErrorBody {
        error: String,
    }
    serde_json::from_slice::<ErrorBody>(body)
        .ok()
        .map(|b| b.error)
}

/// At most 512 bytes of `body` as text, cut on a character boundary.
pub(crate) fn body_excerpt(body: &[u8]) -> String {
    let head = &body[..body.len().min(512)];
    let mut text = String::from_utf8_lossy(head).into_owned();
    // A multi-byte character split by the 512-byte cut decodes to U+FFFD; the
    // excerpt is a human hint, so dropping that last character is fine.
    if head.len() == 512 && text.ends_with('\u{FFFD}') {
        text.pop();
    }
    text
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::*;

    fn now() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_800_000_000)
    }

    #[test]
    fn retry_after_seconds() {
        assert_eq!(parse_retry_after("7", now()), Some(Duration::from_secs(7)));
        assert_eq!(
            parse_retry_after(" 12 ", now()),
            Some(Duration::from_secs(12))
        );
    }

    #[test]
    fn retry_after_http_date_is_relative_to_now() {
        let later = httpdate::fmt_http_date(now() + Duration::from_secs(30));
        assert_eq!(
            parse_retry_after(&later, now()),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn retry_after_past_date_is_zero() {
        let earlier = httpdate::fmt_http_date(now() - Duration::from_secs(30));
        assert_eq!(parse_retry_after(&earlier, now()), Some(Duration::ZERO));
    }

    #[test]
    fn retry_after_garbage_is_none() {
        for v in ["soon", "-5", "", "1.5"] {
            assert_eq!(parse_retry_after(v, now()), None, "{v:?}");
        }
    }

    #[test]
    fn api_message_reads_server_error_body() {
        assert_eq!(
            api_message(br#"{"status":400,"error":"Match is not live"}"#).as_deref(),
            Some("Match is not live")
        );
        assert_eq!(api_message(b"<html>"), None);
        assert_eq!(api_message(br#"{"status":400}"#), None);
    }

    #[test]
    fn body_excerpt_cuts_at_512_bytes_on_a_char_boundary() {
        let s = "x".repeat(511) + "ё" + "tail";
        let e = body_excerpt(s.as_bytes());
        assert!(e.len() <= 512, "{}", e.len());
        assert!(e.is_char_boundary(e.len()));
        assert!(e.starts_with("xxx"));
        assert!(!e.ends_with("tail"));
        assert_eq!(body_excerpt(b"short"), "short");
    }

    #[test]
    fn helpers_follow_the_variant() {
        let url = String::from("https://h/v1/x");
        let rl = Error::RateLimited {
            url: url.clone(),
            retry_after: Some(Duration::from_secs(7)),
        };
        assert_eq!(rl.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(rl.status(), Some(429));
        assert!(!rl.is_not_found());

        let ov = Error::Overloaded {
            url: url.clone(),
            retry_after: None,
        };
        assert_eq!((ov.status(), ov.retry_after()), (Some(503), None));

        let nf = Error::NotFound {
            url: url.clone(),
            message: Some("nope".into()),
        };
        assert!(nf.is_not_found());
        assert_eq!((nf.status(), nf.message()), (Some(404), Some("nope")));

        let st = Error::Status {
            url: url.clone(),
            status: 400,
            message: None,
            body: "{}".into(),
        };
        assert_eq!(
            (st.status(), st.message(), st.retry_after()),
            (Some(400), None, None)
        );

        let tr = Error::Transport {
            url: url.clone(),
            source: "boom".into(),
        };
        assert_eq!(tr.status(), None);

        let de = Error::Decode {
            location: "matches/42/metadata.json".into(),
            source: serde_json::from_str::<u8>("x").unwrap_err(),
        };
        assert_eq!(de.status(), None);
        assert!(de.to_string().contains("matches/42/metadata.json"));
    }
}
