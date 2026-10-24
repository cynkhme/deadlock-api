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

/// A cache that stores each response as one file under a directory.
///
/// The key is the relative path of the file (`matches/42/metadata.json`),
/// so the cache can be inspected by hand. Writes go to a uniquely named
/// temporary file in the target's directory and are renamed into place, so
/// a reader never sees a partial body and two processes writing the same
/// key leave one complete file. Use one directory per server: the key does
/// not contain the host.
///
/// The cache is unbounded and never pruned: only immutable responses are
/// stored, and re-fetching one costs a slot of the server's hourly limit.
/// The directory is the caller's to prune; implement [`Cache`] for a
/// bounded policy.
#[cfg(feature = "fs-cache")]
#[derive(Debug)]
pub struct FileCache {
    dir: std::path::PathBuf,
    counter: std::sync::atomic::AtomicU64,
}

#[cfg(feature = "fs-cache")]
impl FileCache {
    /// A cache rooted at `dir`; the directory is created on the first write.
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// The directory the cache writes into.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    /// The file a key maps to.
    ///
    /// # Errors
    ///
    /// `InvalidInput` when the key is empty, absolute, or has an empty,
    /// `.` or `..` segment: such a key could name a file outside `dir`.
    pub fn path_for(&self, key: &str) -> std::io::Result<std::path::PathBuf> {
        let invalid = || {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid cache key {key:?}"),
            )
        };
        if key.is_empty() || key.starts_with('/') {
            return Err(invalid());
        }
        let mut path = self.dir.clone();
        for segment in key.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(invalid());
            }
            path.push(segment);
        }
        Ok(path)
    }
}

#[cfg(feature = "fs-cache")]
impl Cache for FileCache {
    fn get(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
        match std::fs::read(self.path_for(key)?) {
            Ok(body) => Ok(Some(body)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn put(&self, key: &str, body: &[u8]) -> std::io::Result<()> {
        let path = self.path_for(key)?;
        let parent = path.parent().unwrap_or(&self.dir);
        std::fs::create_dir_all(parent)?;
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();
        let tmp = parent.join(format!("{file_name}.{}.{n}.tmp", std::process::id()));
        if let Err(e) = std::fs::write(&tmp, body) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        std::fs::rename(&tmp, &path).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

#[cfg(all(test, feature = "fs-cache"))]
mod tests {
    use std::io::ErrorKind;
    use std::path::PathBuf;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("deadlock-api-cache-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn put_then_get_roundtrip() {
        let dir = scratch("roundtrip");
        let cache = FileCache::new(&dir);
        cache.put("a.json", b"{\"x\":1}").unwrap();
        assert_eq!(
            cache.get("a.json").unwrap().as_deref(),
            Some(&b"{\"x\":1}"[..])
        );
        cache.put("a.json", b"[]").unwrap();
        assert_eq!(cache.get("a.json").unwrap().as_deref(), Some(&b"[]"[..]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_key_is_none() {
        let dir = scratch("missing");
        let cache = FileCache::new(&dir);
        assert_eq!(cache.get("nope.json").unwrap(), None);
    }

    #[test]
    fn nested_key_creates_parent_dirs() {
        let dir = scratch("nested");
        let cache = FileCache::new(&dir);
        cache.put("matches/42/metadata.json", b"{}").unwrap();
        assert!(dir.join("matches/42/metadata.json").is_file());
        assert_eq!(cache.dir(), dir.as_path());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_temp_file_remains_after_put() {
        let dir = scratch("tmp");
        let cache = FileCache::new(&dir);
        cache.put("matches/42/metadata.json", b"{}").unwrap();
        let names: Vec<String> = std::fs::read_dir(dir.join("matches/42"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["metadata.json"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn key_escaping_dir_is_rejected() {
        let dir = scratch("escape");
        let cache = FileCache::new(&dir);
        for key in ["../x", "/etc/passwd", "a//b", "", "a/../b", ".", "a/./b"] {
            let err = cache.path_for(key).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidInput, "{key:?}");
            assert!(cache.put(key, b"x").is_err(), "{key:?}");
            assert!(cache.get(key).is_err(), "{key:?}");
        }
        assert!(!dir.exists());
    }

    #[test]
    fn two_writers_same_key_leave_one_valid_file() {
        let dir = scratch("race");
        let cache = std::sync::Arc::new(FileCache::new(&dir));
        let handles: Vec<_> = (0..2)
            .map(|i| {
                let cache = cache.clone();
                std::thread::spawn(move || {
                    let body = vec![b'a' + i as u8; 4096];
                    for _ in 0..50 {
                        cache.put("k.json", &body).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let got = cache.get("k.json").unwrap().unwrap();
        assert_eq!(got.len(), 4096);
        assert!(got.iter().all(|&b| b == got[0]));
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["k.json"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
