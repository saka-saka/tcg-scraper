use scraper::Selector;

use crate::domain::LastFetchedAt;

pub(crate) struct OnePieceScraper {}
impl OnePieceScraper {
    pub(crate) async fn set(&self) -> Vec<String> {
        let mut results = vec![];
        let series = "550105";
        let url = format!(
            "https://www.onepiece-cardgame.com/cardlist/?series={}",
            series
        );
        let source = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = scraper::Html::parse_document(&source);
        let option_selector = &Selector::parse("#series option").unwrap();
        for option in document.select(option_selector) {
            let v = option.value().attr("value").unwrap();
            if v.len() != 0 {
                results.push(v.to_string());
            }
        }
        results
    }
    pub(crate) async fn scrape(&self, series: &str) -> Vec<Result<OnePieceCard, ErrorCode>> {
        let mut results = vec![];
        let url = format!(
            "https://www.onepiece-cardgame.com/cardlist/?series={}",
            series
        );
        let source = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = scraper::Html::parse_document(&source);
        let set_name_selector = &Selector::parse("#series option").unwrap();
        let set_name = document
            .select(&set_name_selector)
            .skip_while(|e| e.value().attr("selected").is_none())
            .next()
            .unwrap()
            .inner_html();
        let dls_selector = Selector::parse("div.resultCol dl").unwrap();
        let dls = document.select(&dls_selector);
        for dl in dls {
            let card_name_selector = &Selector::parse("dt .cardName").unwrap();
            let card_name = dl.select(card_name_selector).next().unwrap().inner_html();
            let code_selector = &Selector::parse("dt .infoCol span").unwrap();
            let code = dl.select(code_selector).next().unwrap().inner_html();
            let rarity = dl
                .select(code_selector)
                .skip(1)
                .next()
                .unwrap()
                .inner_html();
            let img_selector = &Selector::parse("dd img").unwrap();
            let img_src = dl
                .select(img_selector)
                .next()
                .unwrap()
                .value()
                .attr("src")
                .unwrap();
            let one_piece_card = OnePieceCard {
                name: card_name,
                code,
                img_src: img_src.to_owned(),
                rarity,
                set_name: set_name.clone(),
                last_fetched_at: LastFetchedAt::default(),
            };
            results.push(Ok(one_piece_card));
        }
        results
    }
}

#[derive(Debug)]
pub struct OnePieceCard {
    pub name: String,
    pub code: String,
    pub img_src: String,
    pub rarity: String,
    pub set_name: String,
    pub last_fetched_at: LastFetchedAt,
}

#[derive(Debug)]
pub enum ErrorCode {
    RarityNotExists,
}
