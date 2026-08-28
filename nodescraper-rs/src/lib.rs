pub mod scrapers;

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/nodescraper.rs"));
}
