use scraper::Selector;

use crate::{
    domain::LastFetchedAt,
    error::{Error, ErrorCode},
};

pub(crate) struct WsScraper {}
impl WsScraper {
    pub async fn get_source(url: &str) -> Result<String, Error> {
        Ok(reqwest::Client::new().get(url).send().await?.text().await?)
    }

    pub async fn get_total_pages(&self) -> Result<i32, Error> {
        let url = "https://ws-tcg.com/cardlist/search";
        let source = Self::get_source(&url).await?;
        let document = scraper::Html::parse_document(&source);
        let selector =
            Selector::parse("#searchResults > p:nth-child(4) > span:nth-child(12) > a").unwrap();
        let mut total_page_selection = document.select(&selector);
        let total_pages: i32 = total_page_selection
            .next()
            .ok_or(Error::FieldMissing("total page not found".to_string()))?
            .inner_html()
            .replace(',', "")
            .parse()?;
        Ok(total_pages)
    }

    pub(crate) async fn scrape_by_page(
        &self,
        page_no: i32,
    ) -> Result<Vec<Result<WsCard, ErrorCode>>, Error> {
        let url = format!("https://ws-tcg.com/cardlist/search?page={}", page_no);
        let source = Self::get_source(&url).await?;
        let document = scraper::Html::parse_document(&source);
        let selector = Selector::parse("table.search-result-table tbody tr").unwrap();
        let trs = document.select(&selector);
        let mut results = vec![];
        for tr in trs {
            let selector = &Selector::parse("h4").unwrap();
            let set_name = tr
                .select(selector)
                .next()
                .ok_or(Error::FieldMissing("set_name not found".to_string()))?
                .text()
                .last()
                .ok_or(Error::FieldMissing("set_name not found".to_string()))?;
            let selector = &Selector::parse("a span").unwrap();
            let mut spans = tr.select(selector);
            let card_name = spans
                .next()
                .ok_or(Error::FieldMissing("card_name not found".to_string()))?;
            let card_no = spans.next().unwrap().inner_html();
            let a = card_no.clone();
            let (set_code, _) = a.split_once('/').unwrap();
            let selector = &Selector::parse("img").unwrap();
            let img_src = tr
                .select(selector)
                .next()
                .ok_or(Error::FieldMissing("img_src not found".to_string()))?
                .value()
                .attr("src")
                .ok_or(Error::FieldMissing("src not found".to_string()))?;
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
                name: card_name.inner_html().trim().to_string(),
                code: card_no,
                set_code: set_code.to_string(),
                img_src: img_src.to_owned(),
                rarity,
                set_name: set_name.to_string().replacen('-', "", 1),
                last_fetched_at,
            };
            results.push(Ok(card))
        }
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct WsCard {
    pub name: String,
    pub code: String,
    pub set_code: String,
    pub img_src: String,
    pub rarity: Option<String>,
    pub set_name: String,
    pub last_fetched_at: LastFetchedAt,
}
