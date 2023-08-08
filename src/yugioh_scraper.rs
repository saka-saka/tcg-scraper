use derive_builder::Builder;
use fantoccini::{wd::Capabilities, ClientBuilder, Locator};
use scraper::Selector;

pub(crate) struct YugiohScraper {
    cap: Capabilities,
    url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not connect to url, forget geckodriver?")]
    NewSession(#[from] fantoccini::error::NewSessionError),
    #[error("could not connect to website")]
    Cmd(#[from] fantoccini::error::CmdError),
    #[error("scraper error")]
    Scraper(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("attribute not found {0}")]
    AttrNotfound(String),
    #[error("card id is not exists")]
    CardIdNotExists,
}

impl YugiohScraper {
    pub(crate) fn new() -> Self {
        let mut cap = Capabilities::new();
        cap.insert(
            "moz:firefoxOptions".to_string(),
            serde_json::json!({"args": ["--headless"]}),
        );
        Self {
            cap,
            url: "http://localhost:4444".to_string(),
        }
    }
    pub async fn fetch_expansion_link(&self) -> Result<Vec<String>, Error> {
        let client = ClientBuilder::native()
            .capabilities(self.cap.clone())
            .connect(&self.url)
            .await?;
        client
            .goto(
                "https://www.db.yugioh-card.com/yugiohdb/card_list.action?clm=1&request_locale=ja",
            )
            .await?;
        client
            .wait()
            .for_element(Locator::Css("#card_list_1 .card_list #list_title_1"))
            .await?;
        let source = client.source().await?;
        let document = scraper::Html::parse_document(&source);
        let selector = &Selector::parse("#card_list_1 .card_list .pack_ja .link_value")?;
        let mut links = vec![];
        for elem in document.select(selector) {
            let v = elem
                .value()
                .attr("value")
                .ok_or_else(|| Error::AttrNotfound(String::from("input value not found")))?;
            links.push(v.to_owned())
        }
        Ok(links)
    }
    pub async fn fetch_printing_link(&self, expansion_link: &str) -> Result<Vec<String>, Error> {
        let client = ClientBuilder::native()
            .capabilities(self.cap.clone())
            .connect(&self.url)
            .await?;
        client.goto(expansion_link).await?;
        client
            .wait()
            .for_element(Locator::Css("#card_list"))
            .await?;
        let source = client.source().await?;
        let document = scraper::Html::parse_document(&source);
        let selector = &Selector::parse("#card_list .t_row.c_normal .link_value")?;
        let mut links = vec![];
        for elem in document.select(selector) {
            let v = elem
                .value()
                .attr("value")
                .ok_or_else(|| Error::AttrNotfound(String::from("input value not found")))?;
            links.push(v.to_owned())
        }
        Ok(links)
    }
    pub async fn fetch_printing_detail(&self, link: &str) -> Result<Vec<YugiohPrinting>, Error> {
        let mut builder = YugiohPrintingBuilder::create_empty();
        let (_, query) = link.split_once("?").ok_or_else(|| Error::CardIdNotExists)?;
        for qs in query.split("&") {
            let (key, value) = qs.split_once("=").unwrap();
            if key == "cid" {
                builder.card_id(value.to_owned());
            }
        }
        if builder.card_id.is_none() {
            return Err(Error::CardIdNotExists);
        }
        let client = ClientBuilder::native()
            .capabilities(self.cap.clone())
            .connect(&self.url)
            .await?;
        client.goto(link).await?;
        client
            .wait()
            .for_element(Locator::Css("#article_body"))
            .await?;
        let source = client.source().await?;
        let document = scraper::Html::parse_document(&source);
        let selector = &Selector::parse("#article_body #cardname")?;
        for elem in document.select(selector) {
            for (i, t) in elem.text().enumerate() {
                if i == 3 {
                    builder.name_jp(t.trim().to_string());
                }
                if i == 4 {
                    builder.name_en(t.to_string());
                }
            }
        }
        let mut printings = vec![];
        for elem in document.select(&Selector::parse("#update_list .t_body .t_row")?) {
            let mut b = builder.clone();
            let release_date = elem
                .select(&Selector::parse(".time").unwrap())
                .last()
                .map(|f| f.inner_html().trim().to_owned())
                .unwrap();
            b.release_date(release_date);
            let number = elem
                .select(&Selector::parse(".card_number").unwrap())
                .last()
                .map(|f| f.inner_html().trim().to_owned())
                .unwrap();
            let (r#ref, number) = match number.split_once("-") {
                Some(r) => r,
                None => ("NONE", "000"),
            };
            b.r#ref(r#ref.to_owned());
            b.number(format!("{}-{number}", r#ref));
            let expansion_name = elem
                .select(&Selector::parse(".pack_name").unwrap())
                .last()
                .map(|f| f.inner_html().trim().to_owned())
                .unwrap();
            b.expansion_name(expansion_name);
            let rarity = elem
                .select(&Selector::parse(".icon p").unwrap())
                .last()
                .map(|f| f.inner_html().trim().to_owned())
                .unwrap();
            b.rarity(rarity);
            let remark = elem
                .select(&Selector::parse(".icon span").unwrap())
                .last()
                .map(|f| f.inner_html().trim().to_owned())
                .unwrap();
            b.remark(remark);
            let printing = b.build().unwrap();
            printings.push(printing)
        }
        Ok(printings)
    }
}

#[derive(Builder, Debug)]
pub struct YugiohPrinting {
    pub card_id: String,
    pub name_jp: String,
    pub name_en: String,
    pub rarity: String,
    pub number: String,
    pub release_date: String,
    pub remark: String,
    pub expansion_name: String,
    pub r#ref: String,
}
