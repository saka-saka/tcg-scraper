use std::str::FromStr;

use scraper::Selector;

use crate::{
    domain::PtcgRarity,
    error::Error,
    scraper::{get_source, Inner},
};

#[derive(Clone)]
pub struct PokemonWikiScraper {}

#[derive(Debug)]
pub struct PokemonWikiCard {
    pub number: String,
    pub name: String,
    pub rarity: PtcgRarity,
    pub exp_code: String,
}

impl PokemonWikiScraper {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn fetch_card_data_by_exp_url(
        &self,
        exp_url: &str,
        exp_code: &str,
    ) -> Result<Vec<PokemonWikiCard>, Error> {
        let source = get_source(exp_url).await?;
        let document = scraper::Html::parse_document(&source);
        let tr_selector =
            &Selector::parse("table > tbody > tr:nth-child(2) > td > table > tbody > tr").unwrap();
        let tr_selection = document.select(tr_selector);
        let mut cards = vec![];
        tr_selection.skip(1).for_each(|tr| {
            let number_selector = &Selector::parse("td:nth-child(1)").unwrap();
            let Some(number) = tr.select(number_selector).next() else {
                return;
            };
            let number = number.inner_trim();
            let name_selector = &Selector::parse("td:nth-child(2) a").unwrap();
            let name = tr.select(name_selector).next().unwrap().inner_trim();

            let rarity_selector_img =
                &Selector::parse("td:nth-child(4) span.explain > img").unwrap();
            let rarity = tr.select(rarity_selector_img).next();
            let rarity = if rarity.is_some() {
                PtcgRarity::A
            } else {
                let rarity_selector = &Selector::parse("td:nth-child(4) span.explain > b").unwrap();

                let rarity = tr
                    .select(rarity_selector)
                    .next()
                    .map(|elem| elem.inner_trim());

                let rarity = rarity.unwrap_or("Unknown".to_string());
                PtcgRarity::from_str(&rarity).unwrap_or(PtcgRarity::Unknown)
            };
            let card = PokemonWikiCard {
                number,
                name,
                rarity,
                exp_code: exp_code.to_string(),
            };
            cards.push(card);
        });
        Ok(cards)
    }
}
