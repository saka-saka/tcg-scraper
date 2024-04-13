use crate::{export_csv::ExportCsv, yugioh_scraper::YugiohPrinting};
use time::macros::format_description;

impl From<YugiohPrinting> for ExportCsv {
    fn from(value: YugiohPrinting) -> Self {
        let now = time::OffsetDateTime::now_utc();
        let format = format_description!("[day]/[month]/[year] [hour]:[minute]");
        Self {
            product_id: None,
            brand: Some(String::from("Yu-Gi-Oh!")),
            set: Some(value.expansion_name),
            edition: None,
            series: None,
            rarity: Some(value.rarity),
            material: None,
            release_year: Some(value.release_date.split_once('-').unwrap().0.to_owned()),
            language: Some(String::from("JP")),
            card_name_english: Some(value.name_en),
            card_name_chinese: None,
            card_name_japanese: Some(value.name_jp),
            card_number: Some(value.number),
            image: None,
            value: None,
            reference: Some(value.r#ref),
            remark: None,
            remark1: Some(now.unix_timestamp().to_string()),
            remark2: Some(now.format(format).unwrap()),
            remark3: None,
            remark4: None,
            remark5: Some(value.remark),
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
