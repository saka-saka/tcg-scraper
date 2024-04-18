use scraper::ElementRef;

use crate::error::Error;

pub mod one_piece;
pub mod ptcg;
pub mod scraper_error;
pub mod tcg_collector;
pub mod ws;
pub mod yugioh;

pub(crate) trait Inner {
    fn inner_trim(&self) -> String;
    fn inner_lowercase_trim(&self) -> String;
}

impl<'a> Inner for ElementRef<'a> {
    fn inner_trim(&self) -> String {
        self.inner_html().trim().to_string()
    }
    fn inner_lowercase_trim(&self) -> String {
        self.inner_html().trim().to_lowercase()
    }
}

pub async fn get_source(url: &str) -> Result<String, Error> {
    Ok(reqwest::Client::new().get(url).send().await?.text().await?)
}
