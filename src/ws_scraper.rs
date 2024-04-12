use scraper::Selector;

use crate::{domain::LastFetchedAt, error::ErrorCode};

pub(crate) struct WsScraper {}
impl WsScraper {
    pub(crate) async fn scrape(&self) -> Vec<Result<WsCard, ErrorCode>> {
        let total_page_count = 1786;
        // let total_page_count = 20;
        let mut results = vec![];
        for page_no in 1..=total_page_count {
            let url = format!("https://ws-tcg.com/cardlist/search?page={}", page_no);
            let source = reqwest::Client::new()
                .get(&url)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            let document = scraper::Html::parse_document(&source);
            let selector = Selector::parse("table.search-result-table tbody tr").unwrap();
            let trs = document.select(&selector);
            for tr in trs {
                let selector = &Selector::parse("h4").unwrap();
                let set_name = tr.select(selector).next().unwrap().text().last().unwrap();
                let selector = &Selector::parse("a span").unwrap();
                let mut spans = tr.select(selector);
                let card_name = spans.next().unwrap();
                let card_no = spans.next().unwrap().inner_html();
                let a = card_no.clone();
                let (set_code, _) = a.split_once("/").unwrap();
                let selector = &Selector::parse("img").unwrap();
                let img_src = tr
                    .select(selector)
                    .next()
                    .unwrap()
                    .value()
                    .attr("src")
                    .unwrap();
                let selector = &Selector::parse("td span").unwrap();
                let mut rarity: Option<String> = None;
                for span in tr.select(selector) {
                    if span.inner_html().contains("レアリティ") {
                        rarity = Some(span.inner_html().replace("レアリティ：", ""));
                        break;
                    }
                }
                let last_fetched_at = LastFetchedAt::default();
                let card = WsCard {
                    name: card_name.inner_html(),
                    code: card_no,
                    series: set_code.to_string(),
                    img_src: img_src.to_owned(),
                    rarity,
                    set_name: set_name.to_string().replacen("-", "", 1),
                    last_fetched_at,
                };
                results.push(Ok(card))
            }
        }
        results
    }
}

#[derive(Debug)]
pub struct WsCard {
    pub name: String,
    pub code: String,
    pub series: String,
    pub img_src: String,
    pub rarity: Option<String>,
    pub set_name: String,
    pub last_fetched_at: LastFetchedAt,
}
