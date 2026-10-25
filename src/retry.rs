//! The retry policy and the decision it makes after each attempt.

use std::time::Duration;

use bytes::Bytes;
use http::{Request, Response};

use crate::Error;
use crate::endpoint::retry_after;

/// When a client sends a request again after a failed attempt.
///
/// A client without a policy sends every request once. With one, it retries
/// a 503, a 429 whose `Retry-After` is at most [`max_wait`](Self::max_wait),
/// and a transport failure that happened before the request reached the
/// server (name resolution or connecting). A timeout after the request was
/// sent is not retried, since the server may already have counted it against
/// the hourly limit; neither is a 429 without `Retry-After`, a 404, any other
/// status or a body that does not decode. A cache hit sends nothing and is
/// never retried. With the built-in transports a failed TLS handshake
/// counts as connecting for the async client, which retries it, and not for
/// the blocking client, which does not. The same split holds for a name
/// lookup that the system resolver fails: the blocking client retries a
/// lookup only when it timed out or found no address. A custom transport's
/// errors are never retried.
///
/// The pause before attempt n + 1 is the larger of the server's
/// `Retry-After` and `base_delay * 2^(n - 1)` capped at `max_delay`. A
/// `Retry-After` above `max_wait` ends the retries at once, so the caller
/// gets the error and decides by [`Error::retry_after`]. There is no jitter:
/// the crate has no random number generator, so clients started together
/// retry together. The async client sleeps on tokio's timer, so its runtime
/// needs the time driver.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "blocking")]
/// # fn main() -> deadlock_api::Result<()> {
/// use std::time::Duration;
///
/// use deadlock_api::RetryPolicy;
///
/// let api = deadlock_api::blocking::Client::builder()
///     .retry(RetryPolicy::new(3).max_wait(Duration::from_secs(30)))
///     .build()?;
/// let health = api.health()?;
/// # let _ = health;
/// # Ok(()) }
/// # #[cfg(not(feature = "blocking"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    max_wait: Duration,
}

impl RetryPolicy {
    /// A policy of at most `max_attempts` attempts in all, the first one
    /// included; 0 counts as 1. The pauses start at 1 s and are capped at
    /// 30 s, and a `Retry-After` up to 60 s is waited out.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            max_wait: Duration::from_secs(60),
        }
    }

    /// Sets the pause before the second attempt; each later pause doubles.
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Sets the cap on the doubling pause.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the longest `Retry-After` the client waits out.
    pub fn max_wait(mut self, wait: Duration) -> Self {
        self.max_wait = wait;
        self
    }

    /// The pause before the next attempt, after `attempts` attempts ended
    /// with `result`; `None` when `result` is final. `unsent` tells whether
    /// a transport error happened before the request reached the server.
    pub(crate) fn pause(
        &self,
        attempts: u32,
        result: &Result<Response<Bytes>, Error>,
        unsent: impl FnOnce(&Error) -> bool,
    ) -> Option<Duration> {
        if attempts >= self.max_attempts {
            return None;
        }
        let backoff = 2u32
            .checked_pow(attempts - 1)
            .map_or(self.max_delay, |factor| {
                self.base_delay.saturating_mul(factor).min(self.max_delay)
            });
        let server_wait = match result {
            Ok(resp) => match resp.status().as_u16() {
                503 => retry_after(resp.headers()).unwrap_or_default(),
                429 => retry_after(resp.headers())?,
                _ => return None,
            },
            Err(e) if unsent(e) => Duration::ZERO,
            Err(_) => return None,
        };
        (server_wait <= self.max_wait).then(|| server_wait.max(backoff))
    }
}

/// A copy of `req` for the next attempt: `http::Request` is not `Clone`.
pub(crate) fn copy_request(req: &Request<()>) -> Request<()> {
    let mut copy = Request::new(());
    *copy.method_mut() = req.method().clone();
    *copy.uri_mut() = req.uri().clone();
    *copy.version_mut() = req.version();
    *copy.headers_mut() = req.headers().clone();
    copy
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    fn response(status: u16, retry_after: Option<&str>) -> Result<Response<Bytes>, Error> {
        let mut resp = Response::builder().status(status);
        if let Some(value) = retry_after {
            resp = resp.header("retry-after", value);
        }
        Ok(resp.body(Bytes::new()).unwrap())
    }

    fn transport_error() -> Result<Response<Bytes>, Error> {
        Err(Error::Transport {
            url: "http://h/".into(),
            source: "boom".into(),
        })
    }

    fn policy() -> RetryPolicy {
        RetryPolicy::new(10)
            .base_delay(100 * MS)
            .max_delay(1000 * MS)
            .max_wait(5000 * MS)
    }

    #[test]
    fn new_counts_every_attempt_and_zero_as_one() {
        let p = RetryPolicy::new(3);
        assert_eq!(p.max_attempts, 3);
        assert_eq!(
            (p.base_delay, p.max_delay, p.max_wait),
            (
                Duration::from_secs(1),
                Duration::from_secs(30),
                Duration::from_secs(60)
            )
        );
        let busy = response(503, None);
        assert!(p.pause(2, &busy, |_| false).is_some());
        assert_eq!(p.pause(3, &busy, |_| false), None);
        assert_eq!(RetryPolicy::new(0).pause(1, &busy, |_| false), None);
    }

    #[test]
    fn backoff_doubles_up_to_max_delay_and_never_overflows() {
        let p = policy();
        let busy = response(503, None);
        let pauses: Vec<_> = (1..=6).map(|n| p.pause(n, &busy, |_| false)).collect();
        let expected = [100, 200, 400, 800, 1000, 1000].map(|ms| Some(ms * MS));
        assert_eq!(pauses, expected);
        let p = RetryPolicy::new(u32::MAX).max_delay(1000 * MS);
        assert_eq!(p.pause(40, &busy, |_| false), Some(1000 * MS));
        assert_eq!(p.pause(u32::MAX - 1, &busy, |_| false), Some(1000 * MS));
    }

    #[test]
    fn retry_after_wins_when_longer_and_never_shortens_the_backoff() {
        let p = policy();
        assert_eq!(
            p.pause(1, &response(503, Some("3")), |_| false),
            Some(3000 * MS)
        );
        assert_eq!(
            p.pause(1, &response(429, Some("0")), |_| false),
            Some(100 * MS)
        );
    }

    #[test]
    fn rate_limit_is_retried_only_with_a_retry_after_within_max_wait() {
        let p = policy();
        assert_eq!(p.pause(1, &response(429, None), |_| false), None);
        assert_eq!(p.pause(1, &response(429, Some("6")), |_| false), None);
        assert_eq!(
            p.pause(1, &response(429, Some("5")), |_| false),
            Some(5000 * MS)
        );
        assert_eq!(p.pause(1, &response(503, Some("6")), |_| false), None);
    }

    #[test]
    fn other_outcomes_are_final() {
        let p = policy();
        for status in [200, 204, 304, 400, 404, 500, 502] {
            assert_eq!(
                p.pause(1, &response(status, None), |_| false),
                None,
                "{status}"
            );
        }
        assert_eq!(p.pause(1, &transport_error(), |_| false), None);
        assert_eq!(p.pause(1, &transport_error(), |_| true), Some(100 * MS));
    }

    #[test]
    fn copy_request_keeps_method_uri_and_headers() {
        let req = Request::builder()
            .method("GET")
            .uri("http://h/v1/x?a=1")
            .header("x-api-key", "k")
            .body(())
            .unwrap();
        let copy = copy_request(&req);
        assert_eq!(copy.method(), req.method());
        assert_eq!(copy.uri(), req.uri());
        assert_eq!(copy.version(), req.version());
        assert_eq!(copy.headers(), req.headers());
    }
}
