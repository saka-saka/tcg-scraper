#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bigweb scraper error")]
    Scraper(#[from] crate::scraper_error::Error),
    #[error("bigweb repository error")]
    Repository(#[from] crate::repository::Error),
    #[error("set is not exist {0}")]
    SetNotExists(String),
    #[error("file write error")]
    FileWrite(#[from] std::io::Error),
}
