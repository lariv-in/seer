//! Duration helpers for Seer runner intervals (stored as seconds).

use lariv_rs::duration::{format_duration, parse_duration};

const NS_PER_SEC: i64 = 1_000_000_000;

/// Parse a human/Go-style duration string into seconds (minimum 1).
pub fn parse_duration_secs(s: &str) -> Result<i64, String> {
    let nanos = parse_duration(s)?;
    Ok((nanos / NS_PER_SEC).max(1))
}

/// Format seconds as a human-readable duration string for form inputs.
pub fn format_duration_secs(secs: i64) -> String {
    format_duration(secs.saturating_mul(NS_PER_SEC))
}
