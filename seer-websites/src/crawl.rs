//! BFS same-origin crawl helpers.

use std::collections::HashSet;
use url::Url;

use crate::ssrf::{normalize_website_url, validate_public_url};

pub fn same_origin(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme() && a.host_str() == b.host_str() && a.port() == b.port()
}

pub fn extract_same_origin_links(html: &str, base: &Url) -> Vec<Url> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let doc = scraper::Html::parse_document(html);
    let Ok(sel) = scraper::Selector::parse("a[href]") else {
        return out;
    };
    for el in doc.select(&sel) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };
        let Ok(joined) = base.join(href) else {
            continue;
        };
        if !same_origin(base, &joined) {
            continue;
        }
        let mut cleaned = joined;
        cleaned.set_fragment(None);
        let key = cleaned.to_string();
        if seen.insert(key) {
            out.push(cleaned);
        }
    }
    out
}

/// Discover same-origin child links when `remaining_depth > 0` (0 = no further hops).
/// Only enqueues URLs that pass SSRF/public-host checks.
pub fn enqueue_discovered(
    html: &str,
    base: &str,
    remaining_depth: u32,
    enqueue: &mut dyn FnMut(String),
) {
    if remaining_depth == 0 {
        return;
    }
    let Ok(base_url) = Url::parse(base) else {
        return;
    };
    for link in extract_same_origin_links(html, &base_url) {
        let Ok(normalized) = normalize_website_url(link.as_str()) else {
            continue;
        };
        if validate_public_url(&normalized).is_err() {
            continue;
        }
        enqueue(normalized.to_string());
    }
}
