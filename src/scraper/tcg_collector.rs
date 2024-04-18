use chrono::NaiveDate;
use scraper::Selector;

use crate::{domain::PtcgRarity, error::Error};

use super::{get_source, Inner};

pub struct TcgCollectorScraper {}

impl TcgCollectorScraper {
    pub async fn fetch_exps(&self) -> Result<Vec<PtcgJpExpansion>, Error> {
        let url = "https://www.tcgcollector.com/expansions/jp?collectionProgressMode=anyCardVariant&releaseDateOrder=newToOld&displayAs=logos";
        let source = get_source(url).await?;
        let document = scraper::Html::parse_document(&source);

        // example: Mar 22, 2024
        let grid_items_sel = Selector::parse(".expansion-logo-grid-item").unwrap();
        let grid_items = document.select(&grid_items_sel);
        let mut exps = vec![];
        for item in grid_items {
            let sel = &Selector::parse(".expansion-logo-grid-item-release-date").unwrap();
            let mut selected_release_date = item.select(sel);
            let release_date = selected_release_date.next().unwrap().inner_trim();
            let d = chrono::NaiveDate::parse_from_str(&release_date, "%b %d, %Y").unwrap();

            let name_sel = &Selector::parse(".expansion-logo-grid-item-expansion-name").unwrap();
            let name_el = item.select(name_sel).next().unwrap();
            let name = name_el.inner_trim();

            let code_sel = &Selector::parse(".expansion-logo-grid-item-expansion-code").unwrap();
            let maybe_code = item
                .select(code_sel)
                .next()
                .map(|e| e.inner_lowercase_trim());

            let Some(code) = maybe_code else {
                continue;
            };
            let link = name_el
                .attr("href")
                .map(|s| format!("https://www.tcgcollector.com{}", s));

            let symbol_sel = &Selector::parse(".expansion-symbol").unwrap();
            let symbol_src = item
                .select(symbol_sel)
                .next()
                .map(|el| el.attr("src").unwrap().to_string());

            let logo_sel = &Selector::parse(".expansion-logo-grid-item-expansion-logo").unwrap();
            let logo_src = item
                .select(logo_sel)
                .next()
                .map(|el| el.attr("src").unwrap().to_string());

            let exp = PtcgJpExpansion {
                name,
                code,
                link,
                symbol_src,
                logo_src,
                release_date: d,
            };
            exps.push(exp);
        }
        Ok(exps)
    }

    pub async fn fetch_card_detail_html(
        &self,
        link: &str,
    ) -> Result<Vec<TcgCollectorCardDetail>, Error> {
        let url = format!("{}?displayAs=list", link);
        let source = get_source(&url).await?;
        let document = scraper::Html::parse_document(&source);

        let exp_code_sel = &Selector::parse("#card-search-result-title-expansion-code").unwrap();
        let exp_code = document
            .select(exp_code_sel)
            .next()
            .unwrap()
            .inner_lowercase_trim();

        let card_items_sel = &Selector::parse(".card-list-item").unwrap();
        let card_items = document.select(card_items_sel);

        let mut cards: Vec<TcgCollectorCardDetail> = vec![];
        for item in card_items {
            let name_sel = &Selector::parse(".card-list-item-entry-text").unwrap();
            let url_path = item.select(name_sel).next().unwrap().attr("href").unwrap();
            let url = format!("https://www.tcgcollector.com{}", url_path);
            let html = get_source(&url).await?;

            let name = item.select(name_sel).next().unwrap().inner_trim();

            let number_sel = &Selector::parse(".card-list-item-card-number > span").unwrap();
            let number = item.select(number_sel).next().unwrap().inner_trim();

            let rarity_sel = &Selector::parse(".card-rarity-symbol").unwrap();
            let rarity = item
                .select(rarity_sel)
                .next()
                .map(|s| s.attr("title").unwrap_or_default())
                .unwrap_or_default();
            let rarity: PtcgRarity = TcgCollectorRarity(&rarity).into();

            let card = TcgCollectorCardDetail {
                name,
                exp_code: exp_code.clone(),
                number,
                html,
                rarity: Some(rarity),
                url: format!("https://www.tcgcollector.com{}", url_path),
            };
            cards.push(card);
        }

        Ok(cards)
    }
    pub async fn fetch_card_rarity(
        &self,
        link: &str,
    ) -> Result<Vec<TcgCollectorCardRarity>, Error> {
        let url = format!("{}?displayAs=list", link);
        let source = get_source(&url).await?;
        let document = scraper::Html::parse_document(&source);

        let card_items_sel = &Selector::parse(".card-list-item").unwrap();
        let card_items = document.select(card_items_sel);

        let mut cards: Vec<TcgCollectorCardRarity> = vec![];
        for item in card_items {
            let name_sel = &Selector::parse(".card-list-item-entry-text").unwrap();
            let url_path = item.select(name_sel).next().unwrap().attr("href").unwrap();
            let url = format!("https://www.tcgcollector.com{}", url_path);

            let rarity_sel = &Selector::parse(".card-rarity-symbol").unwrap();
            let rarity = item
                .select(rarity_sel)
                .next()
                .map(|s| s.attr("title").unwrap_or_default())
                .unwrap_or_default();
            let rarity: PtcgRarity = TcgCollectorRarity(&rarity).into();
            let card = TcgCollectorCardRarity { rarity, url };
            cards.push(card);
        }

        Ok(cards)
    }
    pub async fn fetch_card_detail(
        &self,
        detail: TcgCollectorCardDetail,
    ) -> Result<PtcgJpCard, Error> {
        let document = scraper::Html::parse_document(&detail.html);

        let desc_sel = &Selector::parse("#card-description").unwrap();
        let desc = document.select(desc_sel).next().map(|d| d.inner_trim());

        let skill1_name_en_sel = &Selector::parse(
            "#card-info-body > div.card-attack > div > div.card-attack-header-text > div",
        )
        .unwrap();
        let skill1_name_en = document
            .select(skill1_name_en_sel)
            .next()
            .map(|s| s.inner_trim());

        let skill1_damage_sel = &Selector::parse(
            "#card-info-body > div.card-attack > div > div.card-attack-header-text > span",
        )
        .unwrap();
        let skill1_damage = document
            .select(skill1_damage_sel)
            .next()
            .map(|s| s.inner_trim());

        let card = PtcgJpCard {
            name: detail.name,
            number: detail.number,
            exp_code: detail.exp_code,
            rarity: detail.rarity,
            desc,
            skill1_name_en,
            skill1_damage,
        };
        Ok(card)
    }
}

#[derive(Debug)]
pub struct PtcgJpExpansion {
    pub name: String,
    pub code: String,
    pub link: Option<String>,
    pub symbol_src: Option<String>,
    pub logo_src: Option<String>,
    pub release_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct TcgCollectorCardRarity {
    pub rarity: PtcgRarity,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct TcgCollectorCardDetail {
    pub name: String,
    pub number: String,
    pub exp_code: String,
    pub html: String,
    pub url: String,
    pub rarity: Option<PtcgRarity>,
}

#[derive(Debug, Clone)]
pub struct PtcgJpCard {
    pub name: String,
    pub number: String,
    pub exp_code: String,
    pub desc: Option<String>,
    pub skill1_name_en: Option<String>,
    pub skill1_damage: Option<String>,
    pub rarity: Option<PtcgRarity>,
}

struct TcgCollectorRarity<'a>(&'a str);

impl<'a> From<TcgCollectorRarity<'a>> for PtcgRarity {
    fn from(value: TcgCollectorRarity<'a>) -> Self {
        match value.0 {
            "Ultra Rare (UR)" => PtcgRarity::UR,
            "Shiny Super Rare (SSR)" => PtcgRarity::SSR,
            "ACE SPEC Rare (ACE)" => PtcgRarity::ACE,
            "Hyper Rare (HR)" => PtcgRarity::HR,
            "Super Rare (SR)" => PtcgRarity::SR,
            "Special Art Rare (SAR)" => PtcgRarity::SAR,
            "Character Super Rare (CSR)" => PtcgRarity::CSR,
            "Art Rare (AR)" => PtcgRarity::AR,
            "Character Rare (CHR)" => PtcgRarity::CHR,
            "Shiny (S)" => PtcgRarity::S,
            "Amazing Rare" => PtcgRarity::A,
            "Rare Holo" => PtcgRarity::H,
            "Radiant Rare (K)" => PtcgRarity::K,
            "Promo" => PtcgRarity::PR,
            "Triple Rare (RRR)" => PtcgRarity::RRR,
            "Double Rare (RR)" => PtcgRarity::RR,
            "Rare (R)" => PtcgRarity::R,
            "Uncommon (U)" => PtcgRarity::U,
            "Common (C)" => PtcgRarity::C,
            "Trainer Rare (TR)" => PtcgRarity::TR,
            _ => PtcgRarity::Unknown,
        }
    }
}
