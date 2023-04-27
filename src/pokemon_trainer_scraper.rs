use crate::scraper_error::Error;
use derive_builder::Builder;
use fantoccini::{wd::Capabilities, ClientBuilder, Locator};
use scraper::{ElementRef, Selector};

const POKEMON_TRAINER_SITE_URL_BASE: &str = "https://asia.pokemon-card.com";

#[derive(Debug)]
pub struct ThePTCGSet {
    pub expansion_code: String,
    pub series: String,
    pub name: String,
    pub release_date: String,
}

#[derive(Debug, Builder)]
pub struct ThePTCGCard {
    pub id: String,
    pub kind: String,
    pub evolve_marker: Option<String>,
    pub name: String,
    pub img_src: Option<String>,
    pub hp: Option<String>,
    pub skills: Vec<Skill>,
    pub weak_point: Option<String>,
    pub resist: Option<String>,
    pub escape: Option<String>,
    pub expansion_symbol: Option<String>,
    pub energy: Option<String>,
    pub number: Option<String>,
    pub artist: String,
}

#[derive(Debug, Clone)]
pub struct Skill {
    name: String,
    cost: String,
    damage: String,
    effect: String,
}

pub struct PokemonTrainerSiteScraper {}

impl PokemonTrainerSiteScraper {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {})
    }
    pub async fn fetch_set(&self) -> Result<Vec<ThePTCGSet>, Error> {
        let mut site_url = format!("{}/tw/card-search", POKEMON_TRAINER_SITE_URL_BASE);

        let mut cap = Capabilities::new();
        cap.insert(
            "moz:firefoxOptions".to_string(),
            serde_json::json!({"args": ["--headless"]}),
        );
        let client = ClientBuilder::native()
            .capabilities(cap)
            .connect("http://localhost:4444")
            .await
            .unwrap();
        let mut psets = vec![];
        loop {
            client.goto(&site_url).await.unwrap();
            client
                .wait()
                .for_element(Locator::Css(".expansionList"))
                .await
                .unwrap();
            let source = client.source().await.unwrap();
            let document = scraper::Html::parse_document(&source);
            let expansion_link_selector = &Selector::parse(".expansionLink")
                .map_err(|e| Error::ScraperBackend(e.to_string()))?;
            for link in document.select(expansion_link_selector) {
                let expansion_code = link
                    .value()
                    .attr("href")
                    .unwrap()
                    .split_once("=")
                    .unwrap()
                    .1;
                let series_selector = &Selector::parse(".series").unwrap();
                let series_elem = link.select(series_selector).next().unwrap();
                let expansion_title_selector = &Selector::parse(".expansionTitle").unwrap();
                let expansion_title_elem = link.select(expansion_title_selector).next().unwrap();
                let release_date_selector = &Selector::parse(".relaseDate span").unwrap();
                let release_date_elem = link.select(release_date_selector).next().unwrap();
                let pset = ThePTCGSet {
                    expansion_code: expansion_code.to_owned().to_lowercase(),
                    series: series_elem.inner_html().trim().to_owned(),
                    name: expansion_title_elem.inner_html().trim().to_owned(),
                    release_date: release_date_elem.inner_html().trim().to_owned(),
                };
                psets.push(pset);
            }
            let next_page_link_selector = &Selector::parse("li.paginationItem.next a")
                .map_err(|e| Error::ScraperBackend(e.to_string()))?;
            if let Some(next_page_link) = document.select(next_page_link_selector).next() {
                let href = next_page_link.value().attr("href").unwrap().to_owned();
                site_url = format!("{}{}", POKEMON_TRAINER_SITE_URL_BASE, href);
            } else {
                break;
            }
        }

        Ok(psets)
    }
    pub fn get_fetchables_by_set(&self, set_code: &str) {}
    pub async fn fetch_card_by_id(&self, card_url: &str) {
        let mut cap = Capabilities::new();
        cap.insert(
            "moz:firefoxOptions".to_string(),
            serde_json::json!({"args": ["--headless"]}),
        );
        let client = ClientBuilder::native()
            .capabilities(cap)
            .connect("http://localhost:4444")
            .await
            .unwrap();
        client.goto(card_url).await.unwrap();
        client
            .wait()
            .for_element(Locator::Css(".cardDetailPage"))
            .await
            .unwrap();
        let source = client.source().await.unwrap();
        let mut card_builder = ThePTCGCardBuilder::default();
        let document = scraper::Html::parse_document(&source);
        let common_header =
            get_first_elem_inner_html(".commonHeader", document.root_element()).unwrap();
        if common_header == "招式" {
            card_builder.kind("寶可夢卡".to_string());
        } else {
            card_builder.kind(common_header);
        }
        let page_header_selector = Selector::parse(".pageHeader.cardDetail").unwrap();
        let mut page_header = document
            .select(&page_header_selector)
            .next()
            .unwrap()
            .text();

        // skip first empty string
        page_header.next();
        let evolve_marker = page_header.next().map(|s| s.trim().to_string());
        card_builder.evolve_marker(evolve_marker);
        let name = page_header.next().unwrap().trim();
        card_builder.name(name.to_string());
        let img_selector = Selector::parse(".cardImage img").unwrap();
        let img_elem = document.select(&img_selector).next().unwrap();
        let img_src = img_elem.value().attr("src").unwrap();
        card_builder.img_src(Some(img_src.to_string()));
        let hp =
            get_first_elem_inner_html(".cardInformationColumn .number", document.root_element());
        card_builder.hp(hp);

        let energy_selector = Selector::parse(".mainInfomation img").unwrap();
        let energy = document
            .select(&energy_selector)
            .next()
            .map(|s| s.value().attr("src").unwrap().to_owned());
        card_builder.energy(energy);

        let skill_selector = Selector::parse(".skill").unwrap();
        let skill_elem = document.select(&skill_selector);
        let mut skills = vec![];
        for skill in skill_elem {
            let skill_name = get_first_elem_inner_html(".skillName", skill).unwrap();
            if !skill_name.is_empty() {
                let skill_cost = get_first_elem_inner_html(".skillCost", skill).unwrap();
                let skill_damage = get_first_elem_inner_html(".skillDamage", skill).unwrap();
                let skill_effect = get_first_elem_inner_html(".skillEffect", skill).unwrap();
                let skill = Skill {
                    name: skill_name,
                    cost: skill_cost,
                    damage: skill_damage,
                    effect: skill_effect,
                };
                skills.push(skill);
            }
        }
        card_builder.skills(skills);
        let weak_point = get_first_elem_inner_html(".weakpoint", document.root_element());
        card_builder.weak_point(weak_point);
        let resist = get_first_elem_inner_html(".resist", document.root_element());
        card_builder.resist(resist);
        let escape = get_first_elem_inner_html(".escape", document.root_element());
        card_builder.escape(escape);
        let expansion_symbol =
            get_first_elem_inner_html(".expansionSymbol", document.root_element());
        card_builder.expansion_symbol(expansion_symbol);
        let collector_number =
            get_first_elem_inner_html(".collectorNumber", document.root_element());
        card_builder.number(collector_number);
        let artist = get_first_elem_inner_html(".illustrator a", document.root_element()).unwrap();
        card_builder.artist(artist);
        card_builder.id(card_url.to_string());
        let card = card_builder.build().unwrap();
        println!("{:#?}", card);
    }
}

fn get_first_elem_inner_html(s: &str, elem: ElementRef) -> Option<String> {
    let selector = &Selector::parse(s).unwrap();
    elem.select(selector)
        .next()
        .map(|s| s.inner_html().trim().to_owned())
}
