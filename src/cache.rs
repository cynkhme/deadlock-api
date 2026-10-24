//! Response caching: the [`Cache`] trait and, under `fs-cache`, a file-backed implementation.

/// A byte store keyed by the cache key an endpoint derives from its path.
///
/// Only immutable responses are cached (match metadata in v0.1), so a hit is
/// served without a request. I/O errors from either method are logged and
/// never returned to the caller: a failed read is a miss, a failed write
/// means the next run fetches again.
pub trait Cache: Send + Sync {
    /// The stored body for `key`, or `None` on a miss.
    fn get(&self, key: &str) -> std::io::Result<Option<Vec<u8>>>;
    /// Stores `body` under `key`, replacing any previous value.
    fn put(&self, key: &str, body: &[u8]) -> std::io::Result<()>;
}
