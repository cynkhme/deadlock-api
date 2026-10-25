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
/// The cache never removes anything by itself: only immutable responses are
/// stored, and re-fetching one costs a slot of the server's hourly limit.
/// [`prune_older_than`](Self::prune_older_than) and
/// [`prune_to_size`](Self::prune_to_size) remove entries when called.
/// They treat every `*.json` file under the directory as an entry, so give
/// the cache a directory of its own: a foreign JSON file in it is pruned
/// like a cached response.
#[cfg(feature = "fs-cache")]
#[derive(Debug)]
pub struct FileCache {
    dir: std::path::PathBuf,
}

/// Process-wide, so two caches on one directory never pick the same temp name.
#[cfg(feature = "fs-cache")]
static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "fs-cache")]
impl FileCache {
    /// A cache rooted at `dir`; the directory is created on the first write.
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The directory the cache writes into.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    /// The file a key maps to.
    ///
    /// # Errors
    ///
    /// `InvalidInput` when the key is empty, absolute, or has an empty, `.`
    /// or `..` segment, or a segment containing `\\` or `:`: such a key could
    /// name a file outside `dir` on some platform.
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
            if segment.is_empty()
                || segment == "."
                || segment == ".."
                || segment.contains(['\\', ':'])
            {
                return Err(invalid());
            }
            path.push(segment);
        }
        Ok(path)
    }

    /// Removes every entry written longer ago than `age`, and the temporary
    /// files of writes that failed more than an hour ago.
    ///
    /// Entries are the `*.json` files under the directory, at any depth; a
    /// foreign JSON file in the directory is removed like a cached response.
    /// Symbolic links are neither followed nor removed, and directories are
    /// kept even when empty. A file written while the prune runs may be
    /// removed with the rest. A missing directory prunes nothing.
    ///
    /// # Errors
    ///
    /// The error from listing the directory when it exists but cannot be
    /// read. Failures below it, an unreadable subdirectory or a file that
    /// cannot be removed, are logged and skipped.
    pub fn prune_older_than(&self, age: std::time::Duration) -> std::io::Result<Pruned> {
        let Some(scanned) = self.scan()? else {
            return Ok(Pruned::default());
        };
        let now = std::time::SystemTime::now();
        let mut pruned = Pruned::default();
        pruned.remove_stale_temps(&scanned.temps, now);
        for entry in scanned.entries.iter().filter(|e| e.age(now) > age) {
            pruned.remove(entry);
        }
        Ok(pruned)
    }

    /// Removes the earliest written entries until the entries left take at
    /// most `max_bytes`, and the temporary files of writes that failed more
    /// than an hour ago.
    ///
    /// The order is by write time, not by last read. What counts as an
    /// entry, and what is never touched, is as for
    /// [`prune_older_than`](Self::prune_older_than).
    ///
    /// # Errors
    ///
    /// As for [`prune_older_than`](Self::prune_older_than).
    pub fn prune_to_size(&self, max_bytes: u64) -> std::io::Result<Pruned> {
        let Some(mut scanned) = self.scan()? else {
            return Ok(Pruned::default());
        };
        let mut pruned = Pruned::default();
        pruned.remove_stale_temps(&scanned.temps, std::time::SystemTime::now());
        scanned
            .entries
            .sort_by(|a, b| (a.modified, &a.path).cmp(&(b.modified, &b.path)));
        let mut total: u64 = scanned.entries.iter().map(|e| e.len).sum();
        for entry in &scanned.entries {
            if total <= max_bytes {
                break;
            }
            if pruned.remove(entry) {
                total -= entry.len;
            }
        }
        Ok(pruned)
    }

    /// The entries and temporary files under the directory, or `None` when
    /// it does not exist.
    fn scan(&self) -> std::io::Result<Option<Scanned>> {
        let root = match std::fs::read_dir(&self.dir) {
            Ok(root) => root,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut scanned = Scanned::default();
        let mut subdirs = Vec::new();
        scanned.collect(root, &mut subdirs);
        while let Some(subdir) = subdirs.pop() {
            match std::fs::read_dir(&subdir) {
                Ok(dir) => scanned.collect(dir, &mut subdirs),
                Err(e) => log::warn!("cache prune: cannot list {}: {e}", subdir.display()),
            }
        }
        Ok(Some(scanned))
    }
}

/// What a prune removed.
#[cfg(feature = "fs-cache")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Pruned {
    /// The number of files removed.
    pub files: u64,
    /// Their total size in bytes.
    pub bytes: u64,
}

#[cfg(feature = "fs-cache")]
impl Pruned {
    /// Removes `file` and counts it; false when it could not be removed.
    fn remove(&mut self, file: &CachedFile) -> bool {
        match std::fs::remove_file(&file.path) {
            Ok(()) => {
                self.files += 1;
                self.bytes += file.len;
                true
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::warn!("cache prune: cannot remove {}: {e}", file.path.display());
                }
                false
            }
        }
    }

    fn remove_stale_temps(&mut self, temps: &[CachedFile], now: std::time::SystemTime) {
        for temp in temps.iter().filter(|t| t.age(now) > STALE_TEMP_AGE) {
            self.remove(temp);
        }
    }
}

/// Spec §4: a younger temporary file may belong to a write in progress.
#[cfg(feature = "fs-cache")]
const STALE_TEMP_AGE: std::time::Duration = std::time::Duration::from_secs(60 * 60);

#[cfg(feature = "fs-cache")]
#[derive(Default)]
struct Scanned {
    entries: Vec<CachedFile>,
    temps: Vec<CachedFile>,
}

#[cfg(feature = "fs-cache")]
impl Scanned {
    /// Sorts the files of one directory listing into entries and temporary
    /// files, and queues its subdirectories.
    fn collect(&mut self, dir: std::fs::ReadDir, subdirs: &mut Vec<std::path::PathBuf>) {
        for entry in dir {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("cache prune: cannot list an entry: {e}");
                    continue;
                }
            };
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(meta) => meta,
                Err(e) => {
                    log::warn!("cache prune: cannot stat {}: {e}", path.display());
                    continue;
                }
            };
            if meta.is_dir() {
                subdirs.push(path);
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let files = if name.ends_with(".json") {
                &mut self.entries
            } else if is_temp_name(name) {
                &mut self.temps
            } else {
                continue;
            };
            match meta.modified() {
                Ok(modified) => files.push(CachedFile {
                    path,
                    len: meta.len(),
                    modified,
                }),
                Err(e) => log::warn!("cache prune: no write time for {}: {e}", path.display()),
            }
        }
    }
}

#[cfg(feature = "fs-cache")]
struct CachedFile {
    path: std::path::PathBuf,
    len: u64,
    modified: std::time::SystemTime,
}

#[cfg(feature = "fs-cache")]
impl CachedFile {
    /// Time since the write; zero for a write time in the future.
    fn age(&self, now: std::time::SystemTime) -> std::time::Duration {
        now.duration_since(self.modified).unwrap_or_default()
    }
}

/// The temporary file [`FileCache::put`] writes before renaming it to `file_name`.
#[cfg(feature = "fs-cache")]
fn temp_name(file_name: &str, pid: u32, n: u64) -> String {
    format!("{file_name}.{pid}.{n}.tmp")
}

/// Whether `name` is a [`temp_name`] of a `*.json` file.
#[cfg(feature = "fs-cache")]
fn is_temp_name(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let Some(stem) = name.strip_suffix(".tmp") else {
        return false;
    };
    let mut parts = stem.rsplitn(3, '.');
    let (Some(n), Some(pid), Some(target)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    digits(n) && digits(pid) && target.ends_with(".json")
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
        let n = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();
        let tmp = parent.join(temp_name(&file_name, std::process::id(), n));
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
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

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
        for key in [
            "../x",
            "/etc/passwd",
            "a//b",
            "",
            "a/../b",
            ".",
            "a/./b",
            "a\\..\\b.json",
            "c:x.json",
        ] {
            let err = cache.path_for(key).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidInput, "{key:?}");
            assert!(cache.put(key, b"x").is_err(), "{key:?}");
            assert!(cache.get(key).is_err(), "{key:?}");
        }
        assert!(!dir.exists());
    }

    #[test]
    fn two_instances_on_one_dir_never_collide() {
        let dir = scratch("instances");
        let caches = [
            std::sync::Arc::new(FileCache::new(&dir)),
            std::sync::Arc::new(FileCache::new(&dir)),
        ];
        let handles: Vec<_> = caches
            .into_iter()
            .enumerate()
            .map(|(i, cache)| {
                std::thread::spawn(move || {
                    let body = vec![b'a' + i as u8; 1 << 20];
                    (0..200)
                        .filter(|_| cache.put("k.json", &body).is_err())
                        .count()
                })
            })
            .collect();
        let errors: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(errors, 0);
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

    const HOUR: Duration = Duration::from_secs(60 * 60);

    /// Writes `len` bytes to `dir/rel` with a write time `age` ago.
    fn write_aged(dir: &Path, rel: &str, len: usize, age: Duration) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, vec![b'x'; len]).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(SystemTime::now() - age)
            .unwrap();
    }

    #[test]
    fn prune_of_a_missing_dir_removes_nothing() {
        let cache = FileCache::new(scratch("prune-missing"));
        assert_eq!(
            cache.prune_older_than(Duration::ZERO).unwrap(),
            Pruned::default()
        );
        assert_eq!(cache.prune_to_size(0).unwrap(), Pruned::default());
    }

    #[test]
    fn prune_older_than_removes_only_older_entries_and_keeps_dirs() {
        let dir = scratch("prune-age");
        write_aged(&dir, "matches/1/metadata.json", 10, 2 * HOUR);
        write_aged(&dir, "matches/2/metadata.json", 20, Duration::ZERO);
        let pruned = FileCache::new(&dir).prune_older_than(HOUR).unwrap();
        assert_eq!((pruned.files, pruned.bytes), (1, 10));
        assert!(!dir.join("matches/1/metadata.json").exists());
        assert!(dir.join("matches/1").is_dir());
        assert!(dir.join("matches/2/metadata.json").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_to_size_removes_the_earliest_written_first() {
        let dir = scratch("prune-size");
        write_aged(&dir, "c.json", 100, HOUR);
        write_aged(&dir, "a.json", 100, 3 * HOUR);
        write_aged(&dir, "matches/b.json", 100, 2 * HOUR);
        let cache = FileCache::new(&dir);
        assert_eq!(cache.prune_to_size(300).unwrap(), Pruned::default());
        let pruned = cache.prune_to_size(150).unwrap();
        assert_eq!((pruned.files, pruned.bytes), (2, 200));
        assert!(!dir.join("a.json").exists());
        assert!(!dir.join("matches/b.json").exists());
        assert!(dir.join("c.json").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_removes_only_stale_temp_files_of_put() {
        let dir = scratch("prune-temp");
        write_aged(&dir, "m/x.json.123.4.tmp", 7, 2 * HOUR);
        let kept = [
            "m/y.json.123.5.tmp",
            "m/z.json.abc.1.tmp",
            "m/w.txt.1.2.tmp",
            "notes.txt",
        ];
        write_aged(&dir, kept[0], 1, Duration::ZERO);
        for rel in &kept[1..] {
            write_aged(&dir, rel, 1, 2 * HOUR);
        }
        let pruned = FileCache::new(&dir).prune_to_size(u64::MAX).unwrap();
        assert_eq!((pruned.files, pruned.bytes), (1, 7));
        assert!(!dir.join("m/x.json.123.4.tmp").exists());
        for rel in kept {
            assert!(dir.join(rel).is_file(), "{rel}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn temp_files_of_put_are_recognised() {
        assert!(is_temp_name(&temp_name(
            "metadata.json",
            std::process::id(),
            7
        )));
        assert!(!is_temp_name(&temp_name("metadata.txt", 1, 7)));
    }

    #[cfg(unix)]
    #[test]
    fn prune_does_not_follow_or_remove_symlinks() {
        let dir = scratch("prune-links");
        let outside = scratch("prune-links-outside");
        write_aged(&outside, "old.json", 10, 2 * HOUR);
        std::fs::create_dir_all(&dir).unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("linked")).unwrap();
        std::os::unix::fs::symlink(outside.join("old.json"), dir.join("l.json")).unwrap();
        let pruned = FileCache::new(&dir)
            .prune_older_than(Duration::ZERO)
            .unwrap();
        assert_eq!(pruned, Pruned::default());
        assert!(outside.join("old.json").is_file());
        assert!(dir.join("l.json").symlink_metadata().is_ok());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn prune_of_an_unreadable_dir_is_an_error() {
        let dir = scratch("prune-file");
        std::fs::write(&dir, b"not a directory").unwrap();
        let cache = FileCache::new(&dir);
        assert!(cache.prune_older_than(Duration::ZERO).is_err());
        assert!(cache.prune_to_size(0).is_err());
        let _ = std::fs::remove_file(&dir);
    }
}
