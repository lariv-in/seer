pub const DETAIL_MAP_JS: &str = include_str!("assets/detail_map.js");

pub fn map_style_url() -> String {
    std::env::var("MAP_STYLE_URL").unwrap_or_default()
}
