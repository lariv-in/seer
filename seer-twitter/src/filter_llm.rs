pub async fn filter_posts_llm_stub(filter_text: &str, is_whitelist: bool, title: &str, body: &str) -> bool {
    let filter_text = filter_text.trim();
    if filter_text.is_empty() { return true; }
    let hay = format!("{title}\n{body}").to_lowercase();
    let patterns: Vec<_> = filter_text.lines().map(|l| l.trim().to_lowercase()).filter(|l| !l.is_empty()).collect();
    let matched = patterns.iter().any(|p| hay.contains(p));
    if is_whitelist { matched } else { !matched }
}
