use derive_builder::Builder;
use serde::Deserialize;
use sqlx::types::time::OffsetDateTime;
use std::str::FromStr;
use strum_macros::EnumString;
use time::macros::format_description;

const BIGWEB_POKEMON_URL: &str = "https://www.bigweb.co.jp/ja/products/pokemon/list";

#[derive(Debug)]
pub struct Cardset {
    pub url: CardsetURL,
    pub r#ref: String,
    pub name: String,
    pub result_count: usize,
}

#[derive(Builder, Default, Debug)]
pub struct BigwebScrappedPokemonCard {
    pub id: String,
    pub set_id: String,
    pub name: String,
    pub number: Option<String>,
    pub sale_price: Option<Price>,
    pub rarity: Option<Rarity>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LastFetchedAt {
    pub inner: OffsetDateTime,
}
impl LastFetchedAt {
    pub fn action_code(&self) -> Option<String> {
        let format = format_description!("[year][month][day][hour][minute][second]");
        self.inner.format(format).ok()
    }
    pub fn created_datetime(&self) -> Option<String> {
        let format = format_description!("[day]/[month]/[year] [hour]:[minute]");
        self.inner.format(format).ok()
    }
}

impl Default for LastFetchedAt {
    fn default() -> Self {
        Self {
            inner: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Builder, Default, Debug)]
pub struct PokemonCard {
    pub id: String,
    pub set_id: String,
    pub set_name: String,
    pub set_ref: String,
    pub name: String,
    pub number: Option<String>,
    pub sale_price: Option<i64>,
    pub rarity: Option<String>,
    pub last_fetched_at: LastFetchedAt,
    pub remark: Option<String>,
}

#[derive(Debug)]
pub struct CardsetURL {
    origin_url: url::Url,
    cardset_id: String,
}

#[derive(Deserialize)]
struct QueryParams {
    cardsets: String,
}

impl CardsetURL {
    pub fn from_cardset_id(id: &str) -> Result<Self, Box<Error>> {
        let query_str = &format!("cardsets={}", id);
        let mut parsed_url =
            url::Url::parse(BIGWEB_POKEMON_URL).map_err(|err| Error::Parse(err.to_string()))?;
        parsed_url.set_query(Some(query_str));
        Ok(Self {
            origin_url: parsed_url,
            cardset_id: id.to_string(),
        })
    }
    pub fn parse(url: &str) -> Result<Self, Box<Error>> {
        let parsed_url = url::Url::parse(url).map_err(|err| Error::Parse(err.to_string()))?;
        let query = parsed_url
            .query()
            .ok_or(Error::Parse("query parmas is empty".to_string()))?;
        let query_params: QueryParams =
            serde_qs::from_str(query).map_err(|err| Error::Parse(err.to_string()))?;
        Ok(Self {
            origin_url: parsed_url,
            cardset_id: query_params.cardsets,
        })
    }
    pub fn cardset_id(&self) -> String {
        self.cardset_id.clone()
    }
    pub fn origin_url(&self) -> url::Url {
        self.origin_url.clone()
    }
}

#[derive(Clone, Default, Debug)]
pub struct CardURL {
    product_id: String,
}
impl CardURL {
    pub fn parse(url: &str) -> Result<Self, Box<Error>> {
        let product_id = url
            .split('/')
            .last()
            .ok_or(Error::Parse("malform url".to_string()))?;
        Ok(Self {
            product_id: product_id.to_string(),
        })
    }
    pub fn card_id(&self) -> String {
        self.product_id.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error {0}")]
    Parse(String),
    #[error("Pest Parse error {0}")]
    PestParse(#[from] pest::error::Error<Rule>),
    #[error("LinkTitle builder error {0}")]
    LinkTitleBuilder(#[from] LinkTitleBuilderError),
}

pub struct DescriptionTitle(Option<Rarity>);

#[allow(clippy::upper_case_acronyms)]
#[derive(
    EnumString, strum_macros::Display, Clone, Debug, PartialEq, Default, strum_macros::EnumIter,
)]
pub enum Rarity {
    #[default]
    UR,
    SSR,
    HR,
    SR,
    SAR,
    CSR,
    AR,
    CHR,
    S,
    A,
    H,
    K,
    PR,
    RRR,
    RR,
    R,
    #[strum(to_string = "U", serialize = "UC")]
    U,
    C,
    TR,
    TD,
    #[strum(default)]
    Unknown(String),
}

impl DescriptionTitle {
    pub fn parse(title: &str) -> Self {
        let rarity = title.replace(['[', ']'], "");
        match Rarity::from_str(&rarity) {
            Ok(r) => Self(Some(r)),
            Err(_) => Self(None),
        }
    }
    pub fn rarity(&self) -> Option<Rarity> {
        self.0.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Price(i64);

impl Price {
    pub fn parse(s: &str) -> Result<Self, Box<Error>> {
        let price = s.trim().replace(['円', ','], "");
        let price: i64 = price
            .parse()
            .map_err(|_err| Error::Parse("parse int error".to_string()))?;
        Ok(Self(price))
    }
    pub fn value(&self) -> i64 {
        self.0
    }
}

pub struct ButtonTitle {
    r#ref: String,
    set_name: String,
}

impl ButtonTitle {
    pub fn parse(title: &str) -> Result<Self, Box<Error>> {
        // 【BW3】ヘイルブリザード
        let (left, set_name) = title.split_once('】').ok_or(Error::Parse(format!(
            "button title doesn't have 】, origin title is: {}",
            title
        )))?;
        let r#ref = left.replace("<!----> 【", "").to_lowercase();
        Ok(Self {
            r#ref,
            set_name: set_name.to_string(),
        })
    }
    pub fn r#ref(&self) -> String {
        self.r#ref.clone()
    }
    pub fn set_name(&self) -> String {
        self.set_name.clone()
    }
}

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar.pest"]
struct MyParser;

#[derive(Builder, Debug, Default)]
pub struct LinkTitle {
    raw: String,
    card_name: String,
    promo: Option<String>,
    description: Option<String>,
    trainer_name: Option<String>,
    alter_art: Option<String>,
    special_art: Option<String>,
}

impl LinkTitle {
    #[allow(clippy::result_large_err)]
    pub fn parse(title: &str) -> Result<LinkTitle, Error> {
        let pairs = MyParser::parse(Rule::TITLE, title)?;
        let mut builder = LinkTitleBuilder::default();
        builder.raw(title.to_string());
        builder.promo(None);
        builder.description(None);
        builder.trainer_name(None);
        builder.alter_art(None);
        builder.special_art(None);
        let mut desc = String::new();
        for pair in pairs {
            if pair.as_rule() == Rule::TITLE {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::DESC => {
                            desc.push_str(pair.as_str());
                            &mut builder
                        }
                        Rule::CARD_NAME => builder.card_name(pair.as_str().to_string()),
                        _ => &mut builder,
                    };
                }
            }
        }
        if !desc.is_empty() {
            builder.description(Some(desc));
        }
        builder.build().map_err(|err| err.into())
    }
    pub fn card_name(&self) -> String {
        self.card_name.clone()
    }
    pub fn is_card(&self) -> bool {
        let is_pokemon_card_game = self.raw.contains("ポケモンカードゲーム");
        let is_graded = self.raw.contains("鑑定品");
        !(is_pokemon_card_game || is_graded)
    }
    pub fn remark(&self) -> Option<String> {
        let mut s = String::new();
        if let Some(promo) = &self.promo {
            s.push_str(promo);
        }
        if let Some(desc) = &self.description {
            s.push_str(desc);
        }
        if let Some(trainer_name) = &self.trainer_name {
            s.push_str(trainer_name);
        }
        if let Some(alter_art) = &self.alter_art {
            s.push_str(alter_art)
        }
        if let Some(special_art) = &self.special_art {
            s.push_str(special_art)
        }
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn link_title_parse_ok() {
        LinkTitle::parse("[【SV1a】トリプレットビート]マスカーニャex").unwrap();
    }
    #[test]
    fn link_title_card_name_ok() {
        let link_title = LinkTitle::parse("[【SV1a】トリプレットビート]マスカーニャex").unwrap();
        assert_eq!(link_title.card_name, "マスカーニャex".to_string());
    }
    #[test]
    fn description_title_rarity_ok() {
        let link_title = DescriptionTitle::parse("[UR]");
        assert_eq!(link_title.rarity(), Some(Rarity::UR));
    }
    #[test]
    fn cardset_url_ok() {
        let cardset_url =
            CardsetURL::parse("https://www.bigweb.co.jp/ja/products/pokemon/list?cardsets=7615")
                .unwrap();
        assert_eq!(cardset_url.cardset_id(), "7615".to_string());
    }
    #[test]
    fn cardset_from_id_ok() {
        let cardset_url = CardsetURL::from_cardset_id("7615").unwrap();
        assert_eq!(
            cardset_url.origin_url().to_string(),
            format!("{}?cardsets=7615", BIGWEB_POKEMON_URL)
        );
    }
    #[test]
    fn card_url_ok() {
        let card_url =
            CardURL::parse("https://www.bigweb.co.jp/ja/products/pokemon/cardViewer/3275927")
                .unwrap();
        assert_eq!(card_url.card_id(), "3275927".to_string());
    }
    #[test]
    fn button_title_ref_ok() {
        let button_title = ButtonTitle::parse("<!----> 【BW3】ヘイルブリザード").unwrap();
        assert_eq!(button_title.r#ref(), "bw3".to_string());
    }
    #[test]
    fn button_title_set_name_ok() {
        let button_title = ButtonTitle::parse("<!----> 【BW3】ヘイルブリザード").unwrap();
        assert_eq!(button_title.set_name(), "ヘイルブリザード".to_string());
    }
    #[test]
    fn price_value_ok() {
        let price = Price::parse("1,800円").unwrap();
        assert_eq!(price.value(), 1800);
    }
    #[test]
    fn rarity_ok() {
        let rarity = Rarity::from_str("SSR").unwrap();
        assert_eq!(rarity, Rarity::SSR);
    }
    #[test]
    fn is_card_pokemon_card_game_ok() {
        let link_title = LinkTitle::parse("[【SVAM/SVAL/SVAW】スターターセットex 3種]ポケモンカードゲーム スカーレット＆バイオレット スターターセットex ホゲータ&デンリュウex").unwrap();
        assert!(!link_title.is_card());
    }
    #[test]
    fn is_card_graded_ok() {
        let link_title =
            LinkTitle::parse("[【旧裏プロモ】]ラッキースタジアム【PSA9鑑定品】").unwrap();
        assert!(!link_title.is_card());
    }
}
