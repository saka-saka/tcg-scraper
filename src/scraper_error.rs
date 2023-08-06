use crate::domain::BigwebScrappedPokemonCardBuilderError;
use crate::pokemon_trainer_scraper::ThePTCGCardBuilderError;
use fantoccini::error::{CmdError, NewSessionError};
use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("browser backend error {0}")]
    BrowserBackend(String),
    #[error("scraper backend error {0}")]
    ScraperBackend(String),
    #[error("parse result count error {0}")]
    ParseResultCount(#[from] ParseIntError),
    #[error("ThePTCGCardBuilderError")]
    ThePTCGCardBuilderError(#[from] ThePTCGCardBuilderError),
    #[error("NewSessionError")]
    NewSessionError(#[from] NewSessionError),
    #[error("CmdError")]
    CmdError(#[from] CmdError),
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("data building error {0}")]
    PokemonCardBuilder(#[from] BigwebScrappedPokemonCardBuilderError),
    #[error("link title parsing error {0}")]
    LinkTitleParsing(#[from] crate::domain::Error),
    #[error("fetch cardset error {0}")]
    FetchCardSet(String),
}
