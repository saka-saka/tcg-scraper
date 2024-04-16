use crate::pokemon_trainer_scraper::ThePTCGCardBuilderError;
use fantoccini::error::{CmdError, NewSessionError};
use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("scraper backend error {0}")]
    ScraperBackend(String),
    #[error("parse result count error {0}")]
    ParseResultCount(#[from] ParseIntError),
    #[error("ThePTCGCardBuilderError")]
    ThePTCGCardBuilder(#[from] ThePTCGCardBuilderError),
    #[error("NewSessionError")]
    NewSession(#[from] NewSessionError),
    #[error("CmdError")]
    Cmd(#[from] CmdError),
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
}
