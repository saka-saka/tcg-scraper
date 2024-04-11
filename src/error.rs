#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("scraper parse error {0}")]
    ScraperParse(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("serde_json error {0}")]
    SerdeJson(#[from] serde_json::Error),
}
