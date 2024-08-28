use std::path::Path;

use scraper::Selector;
use serde::Deserialize;
use serde_json::json;
use strum::AsRefStr;

use crate::{
    domain::LastFetchedAt,
    error::{Error, ErrorCode},
};
const BASEURL: &str = "https://www.onepiece-cardgame.com";

pub(crate) struct OnePieceScraper {}
impl OnePieceScraper {
    pub(crate) async fn set(&self) -> Result<Vec<String>, Error> {
        let mut results = vec![];
        let series = "550105";
        let url = format!("{}/cardlist/?series={}", BASEURL, series);
        let source = reqwest::Client::new()
            .get(&url)
            .send()
            .await?
            .text()
            .await?;
        let document = scraper::Html::parse_document(&source);
        let option_selector = &Selector::parse("#series option").unwrap();
        for option in document.select(option_selector) {
            let v = option.value().attr("value").unwrap();
            if !v.is_empty() {
                results.push(v.to_string());
            }
        }
        Ok(results)
    }
    pub(crate) async fn products(&self) -> Result<Vec<OnePieceProduct>, Error> {
        let mut results = vec![];
        let url = format!("{}/products", BASEURL);
        let source = reqwest::Client::new().get(url).send().await?.text().await?;
        let document = scraper::Html::parse_document(&source);
        let selector = &Selector::parse(".productsDetail").unwrap();
        for product_detail in document.select(selector) {
            let selector = &Selector::parse("dd.productsCategory a").unwrap();
            let category = product_detail.select(selector).next().unwrap().inner_html();
            if category != *"BOOSTERS" && category != *"DECKS" {
                continue;
            }
            let selector = &Selector::parse("dt.productsTit span").unwrap();
            let title = product_detail.select(selector).next().unwrap().inner_html();
            let selector = &Selector::parse("dd.productsDate").unwrap();
            let date = product_detail
                .select(selector)
                .next()
                .unwrap()
                .text()
                .nth(2)
                .unwrap();
            results.push(OnePieceProduct {
                title,
                date: date.to_string(),
            });
        }
        Ok(results)
    }
    pub(crate) async fn scrape_cards(
        &self,
        series: &str,
    ) -> Result<Vec<Result<OnePieceCard, ErrorCode>>, Error> {
        let mut results = vec![];
        let url = format!("{}/cardlist/?series={}", BASEURL, series);
        let source = reqwest::Client::new()
            .get(&url)
            .send()
            .await?
            .text()
            .await?;
        let document = scraper::Html::parse_document(&source);
        let set_name_selector = &Selector::parse("#series option").unwrap();
        let set_name = document
            .select(set_name_selector)
            .find(|e| e.value().attr("selected").is_some())
            .unwrap()
            .inner_html();
        let dls_selector = Selector::parse("div.resultCol dl").unwrap();
        let dls = document.select(&dls_selector);
        for dl in dls {
            let get_info_selector = &Selector::parse("dd .getInfo").unwrap();
            let get_info = dl
                .select(get_info_selector)
                .next()
                .unwrap()
                .text()
                .nth(1)
                .unwrap()
                .trim();
            let card_name_selector = &Selector::parse("dt .cardName").unwrap();
            let card_name = dl.select(card_name_selector).next().unwrap().inner_html();
            let code_selector = &Selector::parse("dt .infoCol span").unwrap();
            let rarity = dl.select(code_selector).nth(1).unwrap().inner_html();
            let rarity = serde_json::from_value(json!(&rarity))?;
            let card_type = dl
                .select(code_selector)
                .nth(2)
                .unwrap()
                .inner_html()
                .trim()
                .to_string();
            let card_type: OnePieceCardType = serde_json::from_value(json!(&card_type))?;
            let img_selector = &Selector::parse("dd img").unwrap();
            let img_src = dl
                .select(img_selector)
                .next()
                .unwrap()
                .value()
                .attr("src")
                .unwrap();
            let path = Path::new(img_src);
            let img_src = format!("{}{}", BASEURL, img_src.replace("..", ""));
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let (code, _) = file_name.split_once('.').unwrap();
            let one_piece_card = OnePieceCard {
                name: card_name,
                code: code.to_string(),
                img_src,
                rarity,
                get_info: get_info.to_string(),
                r#type: card_type,
                set_name: set_name.clone(),
                last_fetched_at: LastFetchedAt::default(),
            };
            results.push(Ok(one_piece_card));
        }
        Ok(results)
    }
}

#[derive(Debug)]
pub struct OnePieceCard {
    pub name: String,
    pub code: String,
    pub img_src: String,
    pub rarity: OnePieceCardRarity,
    pub set_name: String,
    pub r#type: OnePieceCardType,
    pub last_fetched_at: LastFetchedAt,
    pub get_info: String,
}

#[derive(Debug, Deserialize, sqlx::Type)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "op_type_enum")]
pub enum OnePieceCardType {
    Leader,
    #[serde(alias = "事件")]
    Event,
    #[serde(alias = "キャラ")]
    Character,
    Stage,
}

#[derive(Debug, Deserialize, sqlx::Type, AsRefStr)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "op_rarity_enum")]
pub enum OnePieceCardRarity {
    #[serde(alias = "SP卡", alias = "SPカード")]
    SP,
    R,
    #[allow(clippy::upper_case_acronyms)]
    SEC,
    C,
    P,
    UC,
    SR,
    L,
}

#[derive(Debug)]
pub struct OnePieceProduct {
    pub title: String,
    pub date: String,
}
