use crate::domain::{
    BigwebScrappedPokemonCard, BigwebScrappedPokemonCardBuilder,
    BigwebScrappedPokemonCardBuilderError, ButtonTitle, CardURL, Cardset, CardsetURL,
    DescriptionTitle, LinkTitle, Price,
};
use headless_chrome::{Browser, LaunchOptions};
use scraper::Selector;
use std::{num::ParseIntError, thread::sleep, time::Duration};

const BIGWEB_POKEMON_URL: &str =
    "https://www.bigweb.co.jp/ja/products/%E3%83%9D%E3%82%B1%E3%83%A2%E3%83%B3/list?cardsets=7615";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("browser backend error {0}")]
    BrowserBackend(String),
    #[error("scraper backend error {0}")]
    ScraperBackend(String),
    #[error("parse result count error {0}")]
    ParseResultCount(#[from] ParseIntError),
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

pub struct BigwebScraper {
    browser: Browser,
}

impl BigwebScraper {
    pub fn new() -> Result<Self, Error> {
        let mut builder = LaunchOptions::default_builder();
        let option = builder
            .idle_browser_timeout(Duration::MAX)
            .build()
            .map_err(|err| Error::BrowserBackend(err.to_string()))?;
        let browser = Browser::new(option).map_err(|err| Error::BrowserBackend(err.to_string()))?;
        Ok(Self { browser })
    }
    pub fn fetch_pokemon_cardset(&self) -> Result<Vec<Result<Cardset, DataError>>, Error> {
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        tab.navigate_to(BIGWEB_POKEMON_URL)
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let cardset_list = tab
            .wait_for_element("app-cardset-list")
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let content = cardset_list
            .get_content()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let app_cardset_list_fragment = scraper::Html::parse_fragment(&content);
        let selector = &Selector::parse("button.cardset-list-name")
            .map_err(|e| Error::ScraperBackend(e.to_string()))?;
        let buttons = app_cardset_list_fragment.select(selector);
        let mut cardsets = vec![];
        for (i, button) in buttons.enumerate() {
            if let Err(err) = cardset_list.call_js_fn(
                "function (a) { document.querySelectorAll('button.cardset-list-name')[a].click() }",
                vec![serde_json::Value::Number(
                    serde_json::Number::from_f64(i as f64).unwrap(),
                )],
                false,
            ) {
                let err = DataError::FetchCardSet(err.to_string());
                cardsets.push(Err(err));
                continue;
            }

            // wait for url that correspond to the button
            let result_count_elem = match tab.wait_for_element(".result_count") {
                Ok(r) => r,
                Err(err) => {
                    let err_message = format!("wait_for_element `.result_count` but {}", err);
                    let err = DataError::FetchCardSet(err_message);
                    cardsets.push(Err(err));
                    continue;
                }
            };
            let result_count_text = result_count_elem
                .get_inner_text()
                .map_err(|e| Error::BrowserBackend(e.to_string()))?;

            let result_count = result_count_text
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>();

            let result_count: usize = result_count.parse()?;

            let cardset_url = CardsetURL::parse(&tab.get_url()).unwrap();
            let button_title = match ButtonTitle::parse(&button.inner_html()) {
                Ok(t) => t,
                Err(_) => continue,
            };
            cardsets.push(Ok(Cardset {
                url: cardset_url,
                r#ref: button_title.r#ref(),
                name: button_title.set_name(),
                result_count,
            }));
        }
        Ok(cardsets)
    }
    pub fn fetch_pokemon_data(
        &self,
        url: &str,
    ) -> Result<Vec<Result<BigwebScrappedPokemonCard, DataError>>, Error> {
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        tab.navigate_to(url)
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let result_count_element = tab
            .wait_for_element(".product-list")
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let raw_html = tab
            .get_content()
            .map_err(|e| Error::BrowserBackend(e.to_string()))?;
        let mut document = scraper::Html::parse_document(&raw_html);
        let selector =
            Selector::parse(".result_count").map_err(|e| Error::ScraperBackend(e.to_string()))?;
        let result_count = document.select(&selector);
        let inner_html = match result_count.last() {
            Some(elem) => elem.inner_html(),
            None => return Ok(vec![]),
        };

        let result_count = inner_html
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        let result_count: usize = result_count.parse()?;
        let selector =
            Selector::parse(".item-box").map_err(|e| Error::ScraperBackend(e.to_string()))?;
        let item_boxes = document.select(&selector);
        let mut count = item_boxes.count();
        let mut loop_count = 0;
        loop {
            loop_count += 1;
            if loop_count >= 10 {
                let error_message =
                    format!("item count({count}) doesn't meet result_count({result_count})");
                return Err(Error::ScraperBackend(error_message));
            }
            if count == result_count {
                break;
            }
            result_count_element
                .call_js_fn(
                    "function () { window.scrollTo(0, document.body.scrollHeight) }",
                    vec![],
                    false,
                )
                .map_err(|e| Error::BrowserBackend(e.to_string()))?;
            sleep(Duration::from_secs(10));
            let raw_html = tab
                .get_content()
                .map_err(|e| Error::BrowserBackend(e.to_string()))?;
            document = scraper::Html::parse_document(&raw_html);
            let selector =
                Selector::parse(".item-box").map_err(|e| Error::ScraperBackend(e.to_string()))?;
            let item_boxes = document.select(&selector);
            count = item_boxes.count();
        }

        let selector =
            Selector::parse(".item-box").map_err(|e| Error::ScraperBackend(e.to_string()))?;
        let item_boxes = document.select(&selector);
        let mut data = vec![];
        let cardset_url = CardsetURL::parse(url).unwrap();
        for item_box in item_boxes {
            let selector = Selector::parse(".item-point").unwrap();
            if let Some(item_point) = item_box.select(&selector).next() {
                if !item_point.inner_html().replace("<!---->", "").is_empty() {
                    continue;
                }
            }
            let mut builder = BigwebScrappedPokemonCardBuilder::default();
            builder.set_id(cardset_url.cardset_id());
            if let Some(name) = item_box
                .select(&Selector::parse(".images-item-title a").unwrap())
                .next()
            {
                let inner_html = name.inner_html();
                let decoded_html = html_escape::decode_html_entities(&inner_html);
                let href = name.value().attr("href").unwrap();
                let link_title = match LinkTitle::parse(decoded_html.trim()) {
                    Ok(l) => l,
                    Err(err) => {
                        data.push(Err(err.into()));
                        continue;
                    }
                };
                let card_url = CardURL::parse(href).unwrap();
                builder.id(card_url.card_id());
                builder.name(link_title.card_name());
                builder.remark(link_title.remark());
            }

            let number_title_selector = Selector::parse(".grid-item-comment").unwrap();
            match item_box.select(&number_title_selector).next() {
                Some(number_title) => {
                    let inner_html = number_title.inner_html();
                    builder.number(Some(inner_html));
                }
                None => {
                    builder.number(None);
                }
            }

            let selector = Selector::parse(".images-item-title span:nth-of-type(2)")
                .map_err(|e| Error::ScraperBackend(e.to_string()))?;
            match item_box.select(&selector).next() {
                Some(title) => {
                    let desc_title = DescriptionTitle::parse(&title.inner_html());
                    builder.rarity(desc_title.rarity());
                }
                None => {
                    builder.rarity(None);
                }
            }
            let selector = Selector::parse(".sales-price")
                .map_err(|e| Error::ScraperBackend(e.to_string()))?;
            match item_box.select(&selector).next() {
                Some(price) => {
                    let price = price.inner_html();
                    let price = Price::parse(&price).ok();
                    builder.sale_price(price);
                }
                None => {
                    builder.sale_price(None);
                }
            }
            data.push(builder.build().map_err(|err| err.into()))
        }
        Ok(data)
    }
}
