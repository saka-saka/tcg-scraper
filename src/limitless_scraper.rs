use scraper::Selector;

pub(crate) struct LimitlessScraper {}

impl LimitlessScraper {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn poc(&self) {
        // let url = "https://limitlesstcg.com/cards/jp";
        let url = "https://limitlesstcg.com/cards/jp/SV4K/2?translate=en";
        let source = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = scraper::Html::parse_document(&source);
        // section.card-page-main .card-details .card-text-name a
        let selector =
            &Selector::parse("section.card-page-main .card-details .card-text-name a").unwrap();
        // let mut links = vec![];
        let elem = document.select(selector).next().unwrap();
        println!("{}", elem.inner_html());
        // for elem in document.select(selector) {
        //     let v = elem
        //         .value()
        //         .attr("value")
        //         .ok_or_else(|| Error::AttrNotfound(String::from("input value not found")))
        //         .unwrap();
        //     links.push(v.to_owned())
        // }
    }
}

// "https://limitlesstcg.com/cards/jp"
