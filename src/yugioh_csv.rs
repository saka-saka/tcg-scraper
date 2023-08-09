use serde::Serialize;
use time::macros::format_description;

use crate::yugioh_scraper::YugiohPrinting;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct YugiohCsv {
    product_id: Option<String>,
    brand: String,
    set: String,
    edition: Option<String>,
    series: Option<String>,
    rarity: String,
    material: Option<String>,
    release_year: String,
    language: String,
    card_name_english: String,
    card_name_chinese: Option<String>,
    card_name_japanese: String,
    card_number: String,
    image: Option<String>,
    value: Option<String>,
    reference: String,
    remark: Option<String>,
    remark1: Option<String>,
    remark2: Option<String>,
    remark3: Option<String>,
    remark4: Option<String>,
    remark5: String,
    remark6: Option<String>,
    remark7: Option<String>,
    remark8: Option<String>,
    remark9: Option<String>,
    remark10: Option<String>,
    enable: Option<String>,
    #[serde(rename(serialize = "P_Language"))]
    p_language: Option<String>,
    #[serde(rename(serialize = "id"))]
    id: Option<String>,
}

impl From<YugiohPrinting> for YugiohCsv {
    fn from(value: YugiohPrinting) -> YugiohCsv {
        let now = time::OffsetDateTime::now_utc();
        let format = format_description!("[day]/[month]/[year] [hour]:[minute]");
        YugiohCsv {
            product_id: None,
            brand: String::from("Yu-Gi-Oh!"),
            set: value.expansion_name,
            edition: None,
            series: None,
            rarity: value.rarity,
            material: None,
            release_year: value.release_date.split_once('-').unwrap().0.to_owned(),
            language: String::from("JP"),
            card_name_english: value.name_en,
            card_name_chinese: None,
            card_name_japanese: value.name_jp,
            card_number: value.number,
            image: None,
            value: None,
            reference: value.r#ref,
            remark: None,
            remark1: Some(now.unix_timestamp().to_string()),
            remark2: Some(now.format(format).unwrap()),
            remark3: None,
            remark4: None,
            remark5: value.remark,
            remark6: None,
            remark7: None,
            remark8: None,
            remark9: None,
            remark10: None,
            enable: None,
            p_language: None,
            id: None,
        }
    }
}
