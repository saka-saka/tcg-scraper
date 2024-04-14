use std::num::ParseIntError;

use crate::repository::RepositoryError;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("scraper parse error {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("file error {0}")]
    File(#[from] std::io::Error),
    #[error("repository error {0}")]
    Repository(#[from] RepositoryError),
    #[error("url parse error {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("bigweb scraper error {0}")]
    Scraper(#[from] crate::scraper_error::Error),
    #[error("set is not exist {0}")]
    SetNotExists(String),
    #[error("field missing {0}")]
    FieldMissing(String),
    #[error("ParseInt error {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("csv error {0}")]
    Csv(#[from] csv::Error),
}

#[derive(Debug)]
pub enum ErrorCode {
    RarityNotExists,
}
