use crate::messages;

pub mod website;

pub trait Scraper {
    type Error;

    type Args: prost::Message;

    fn init() -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn scrape(
        &self,
        command_id: u64,
        trigger: Self::Args,
    ) -> impl Future<Output = messages::Response>;
}
