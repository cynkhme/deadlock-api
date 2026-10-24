//! Client configuration shared by the blocking and the async client.

use std::sync::Arc;
use std::time::Duration;

use crate::{Cache, Error};

const DEFAULT_BASE_URL: &str = "https://api.deadlock-api.com";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// The settings a client was built with.
///
/// Built by [`ConfigBuilder`]; every client exposes its copy through a
/// `config()` method.
#[derive(Clone)]
pub struct Config {
    base_url: String,
    api_key: Option<String>,
    user_agent: String,
    connect_timeout: Duration,
    timeout: Duration,
    cache: Option<Arc<dyn Cache>>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<set>"))
            .field("user_agent", &self.user_agent)
            .field("connect_timeout", &self.connect_timeout)
            .field("timeout", &self.timeout)
            .field("cache", &self.cache.as_ref().map(|_| "<set>"))
            .finish()
    }
}

impl Config {
    /// The server root without a trailing slash, `https://api.deadlock-api.com` by default.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The API key sent as the `X-API-KEY` header, if any.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// The `User-Agent` header, `deadlock-api-rs/<crate version>` by default.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// The connection timeout, 10 seconds by default.
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// The whole-request timeout, 60 seconds by default.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The cache consulted for cacheable endpoints, if one was set.
    pub fn cache(&self) -> Option<&dyn Cache> {
        self.cache.as_deref()
    }
}

/// Builder for [`Config`]; the client builders expose the same setters.
#[derive(Default)]
pub struct ConfigBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    user_agent: Option<String>,
    connect_timeout: Option<Duration>,
    timeout: Option<Duration>,
    cache: Option<Arc<dyn Cache>>,
}

impl std::fmt::Debug for ConfigBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigBuilder")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<set>"))
            .field("user_agent", &self.user_agent)
            .field("connect_timeout", &self.connect_timeout)
            .field("timeout", &self.timeout)
            .field("cache", &self.cache.as_ref().map(|_| "<set>"))
            .finish()
    }
}

impl ConfigBuilder {
    /// Sets the server root; trailing slashes are removed.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets the API key sent as the `X-API-KEY` header; `None` sends no key.
    pub fn api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }

    /// Sets the `User-Agent` header.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sets the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Sets the whole-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the cache consulted for cacheable endpoints.
    pub fn cache(mut self, cache: impl Cache + 'static) -> Self {
        self.cache = Some(Arc::new(cache));
        self
    }

    /// Resolves defaults, validates the settings and produces the [`Config`].
    ///
    /// # Errors
    ///
    /// [`Error::Config`] when the base URL has no scheme or host, or when the
    /// user agent or the API key is not a valid HTTP header value (for
    /// example an API key read from a file with its trailing newline).
    pub fn build(self) -> crate::Result<Config> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned())
            .trim_end_matches('/')
            .to_owned();
        let uri: http::Uri = base_url.parse().map_err(|e| Error::Config {
            message: format!("base_url {base_url:?}: {e}"),
        })?;
        if uri.scheme().is_none() || uri.authority().is_none() {
            return Err(Error::Config {
                message: format!("base_url {base_url:?} needs a scheme and a host"),
            });
        }
        let user_agent = self
            .user_agent
            .unwrap_or_else(|| format!("deadlock-api-rs/{}", env!("CARGO_PKG_VERSION")));
        header_value("user_agent", &user_agent)?;
        if let Some(key) = &self.api_key {
            header_value("api_key", key)?;
        }
        Ok(Config {
            base_url,
            api_key: self.api_key,
            user_agent,
            connect_timeout: self.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT),
            timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            cache: self.cache,
        })
    }
}

fn header_value(setting: &str, value: &str) -> crate::Result<()> {
    http::HeaderValue::from_str(value)
        .map(drop)
        .map_err(|e| Error::Config {
            message: format!("{setting} is not a valid header value: {e}"),
        })
}

/// Emits the six [`ConfigBuilder`] setters on a client builder whose
/// `config: ConfigBuilder` field holds the settings.
macro_rules! builder_setters {
    ($t:ty) => {
        impl $t {
            /// Sets the server root; trailing slashes are removed.
            pub fn base_url(mut self, url: impl Into<String>) -> Self {
                self.config = self.config.base_url(url);
                self
            }

            /// Sets the API key sent as the `X-API-KEY` header; `None` sends no key.
            pub fn api_key(mut self, key: Option<String>) -> Self {
                self.config = self.config.api_key(key);
                self
            }

            /// Sets the `User-Agent` header.
            pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
                self.config = self.config.user_agent(user_agent);
                self
            }

            /// Sets the connection timeout.
            pub fn connect_timeout(mut self, timeout: ::std::time::Duration) -> Self {
                self.config = self.config.connect_timeout(timeout);
                self
            }

            /// Sets the whole-request timeout.
            pub fn timeout(mut self, timeout: ::std::time::Duration) -> Self {
                self.config = self.config.timeout(timeout);
                self
            }

            /// Sets the cache consulted for cacheable endpoints.
            pub fn cache(mut self, cache: impl $crate::Cache + 'static) -> Self {
                self.config = self.config.cache(cache);
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn defaults_are_the_documented_values() {
        let c = ConfigBuilder::default().build().unwrap();
        assert_eq!(c.base_url(), "https://api.deadlock-api.com");
        assert_eq!(c.connect_timeout(), Duration::from_secs(10));
        assert_eq!(c.timeout(), Duration::from_secs(60));
        assert!(
            c.user_agent().starts_with("deadlock-api-rs/"),
            "{}",
            c.user_agent()
        );
        assert_eq!(c.api_key(), None);
        assert!(c.cache().is_none());
    }

    #[test]
    fn trailing_slash_is_stripped() {
        let c = ConfigBuilder::default()
            .base_url("https://h/")
            .build()
            .unwrap();
        assert_eq!(c.base_url(), "https://h");
        let c = ConfigBuilder::default()
            .base_url("https://h///")
            .build()
            .unwrap();
        assert_eq!(c.base_url(), "https://h");
    }

    #[test]
    fn setters_are_stored() {
        struct NoCache;
        impl crate::Cache for NoCache {
            fn get(&self, _: &str) -> std::io::Result<Option<Vec<u8>>> {
                Ok(None)
            }
            fn put(&self, _: &str, _: &[u8]) -> std::io::Result<()> {
                Ok(())
            }
        }
        let c = ConfigBuilder::default()
            .api_key(Some("k".into()))
            .user_agent("ua/1")
            .connect_timeout(Duration::from_millis(5))
            .timeout(Duration::from_millis(7))
            .cache(NoCache)
            .build()
            .unwrap();
        assert_eq!(c.api_key(), Some("k"));
        assert_eq!(c.user_agent(), "ua/1");
        assert_eq!(c.connect_timeout(), Duration::from_millis(5));
        assert_eq!(c.timeout(), Duration::from_millis(7));
        assert!(c.cache().is_some());
    }

    #[test]
    fn build_rejects_invalid_settings() {
        let cases: [(&str, ConfigBuilder); 4] = [
            ("user_agent", ConfigBuilder::default().user_agent("dlhl\n")),
            (
                "api_key",
                ConfigBuilder::default().api_key(Some("key\n".into())),
            ),
            ("base_url", ConfigBuilder::default().base_url("not a url")),
            (
                "base_url",
                ConfigBuilder::default().base_url("api.deadlock-api.com"),
            ),
        ];
        for (setting, builder) in cases {
            match builder.build() {
                Err(Error::Config { message }) => assert!(message.contains(setting), "{message}"),
                other => panic!("{setting}: {other:?}"),
            }
        }
    }
}
