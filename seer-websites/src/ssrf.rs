//! SSRF validation for website fetch URLs.

use std::net::ToSocketAddrs;
use url::Url;

fn is_public_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast())
        }
        std::net::IpAddr::V6(v6) => {
            !(v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() || v6.is_unique_local())
        }
    }
}

pub fn normalize_website_url(raw: &str) -> anyhow::Result<Url> {
    let raw = html_unescape(raw.trim());
    if raw.is_empty() {
        anyhow::bail!("url is required");
    }
    let mut parsed = Url::parse(&raw).map_err(|e| anyhow::anyhow!("invalid url: {e}"))?;
    if parsed.host().is_none() || parsed.scheme().is_empty() {
        anyhow::bail!("url must be absolute http(s)");
    }
    if parsed.set_username("").is_err() {
        tracing::warn!(url = %parsed, "failed to clear url username");
    }
    if parsed.set_password(None).is_err() {
        tracing::warn!(url = %parsed, "failed to clear url password");
    }
    match parsed.scheme() {
        "http" | "https" => {}
        _ => anyhow::bail!("url must use http or https"),
    }
    Ok(parsed)
}

fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub fn validate_public_url(url: &Url) -> anyhow::Result<()> {
    let host = url.host_str().unwrap_or("").to_lowercase();
    if host.is_empty() || host == "localhost" {
        anyhow::bail!("blocked host");
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if !is_public_ip(&ip) {
            anyhow::bail!("blocked IP");
        }
        return Ok(());
    }
    let addrs = format!("{host}:{}", url.port_or_known_default().unwrap_or(80))
        .to_socket_addrs()
        .map_err(|e| anyhow::anyhow!("dns: {e}"))?
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        anyhow::bail!("dns empty");
    }
    for a in addrs {
        if !is_public_ip(&a.ip()) {
            anyhow::bail!("blocked resolved IP");
        }
    }
    Ok(())
}
