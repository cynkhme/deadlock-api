//! Patch notes routes. Dates stay strings as the server sends them (RFC 3339
//! for `pub_date` and the big days): the crate has no date dependency.

use serde::{Deserialize, Serialize};

/// A patch note from the forum's changelog feed, `GET /v1/patches`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Patch {
    /// The post title.
    pub title: String,
    /// The publication time, RFC 3339.
    pub pub_date: String,
    /// The post URL.
    pub link: String,
    /// The feed's id of the post.
    pub guid: Guid,
    /// The post author.
    pub author: Option<String>,
    /// The forum section.
    pub category: Option<Category>,
    /// The author as the feed's Dublin Core `creator`.
    pub dc_creator: Option<String>,
    /// The post body, HTML.
    pub content_encoded: Option<String>,
    /// The number of comments, a decimal string as the server sends it.
    pub slash_comments: Option<String>,
}

/// A patch note from the combined feed of the forum and Steam news,
/// `GET /v2/patches`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FeedItem {
    /// Where the note was posted; observed values in the 2026-10-25 sample:
    /// `forum`, `steam`.
    pub source: String,
    /// The post title.
    pub title: String,
    /// The publication time, RFC 3339.
    pub pub_date: String,
    /// The post URL.
    pub link: String,
    /// The feed's id of the post.
    pub guid: Guid,
    /// The forum section; only forum posts carry one.
    pub category: Option<Category>,
    /// The post body, HTML.
    pub content: Option<String>,
}

/// The id of a feed entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Guid {
    /// Whether `text` is a permanent URL of the post.
    pub is_perma_link: Option<bool>,
    /// The id, a URN or a URL.
    pub text: String,
}

/// The section a feed entry was posted in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Category {
    /// The section URL.
    pub domain: String,
    /// The section name.
    pub text: String,
}
