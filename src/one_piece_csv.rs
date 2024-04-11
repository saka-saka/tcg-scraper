use crate::one_piece_scraper::{OnePieceCard, OnePieceProduct};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OnePieceCsv {
    product_id: Option<String>,
    brand: Option<String>,
    set: Option<String>,
    edition: Option<String>,
    series: Option<String>,
    rarity: Option<String>,
    material: Option<String>,
    release_year: Option<String>,
    language: Option<String>,
    card_name_english: Option<String>,
    card_name_chinese: Option<String>,
    card_name_japanese: Option<String>,
    card_number: Option<String>,
    image: Option<String>,
    value: Option<String>,
    reference: Option<String>,
    remark: Option<String>,
    remark1: Option<String>,
    remark2: Option<String>,
    remark3: Option<String>,
    remark4: Option<String>,
    remark5: Option<String>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OnePieceProductsCsv {
    title: String,
    date: String,
}

impl From<OnePieceProduct> for OnePieceProductsCsv {
    fn from(value: OnePieceProduct) -> Self {
        OnePieceProductsCsv {
            title: sanitize(&value.title),
            date: value.date,
        }
    }
}

impl From<OnePieceCard> for OnePieceCsv {
    fn from(value: OnePieceCard) -> Self {
        let code = value.code.clone();
        let (set_code, card_number) = code.split_once('-').unwrap();
        let reference = Some(set_code.to_owned());
        let remark9 = Some(set_code.to_owned());
        OnePieceCsv {
            product_id: None,
            brand: Some(String::from("One Piece")),
            set: Some(sanitize(&value.set_name)),
            edition: None,
            series: None,
            rarity: Some(value.rarity.as_ref().to_string()),
            material: None,
            release_year: None,
            language: Some(String::from("ja")),
            card_name_english: None,
            card_name_chinese: None,
            card_name_japanese: Some(value.name),
            card_number: Some(value.code.clone()),
            image: Some(value.img_src),
            value: None,
            reference,
            remark: None,
            remark1: value.last_fetched_at.action_code(),
            remark2: value.last_fetched_at.created_datetime(),
            remark3: None,
            remark4: Some(value.code),
            remark5: None,
            remark6: None,
            remark7: None,
            remark8: None,
            remark9,
            remark10: None,
            enable: None,
            p_language: None,
            id: None,
        }
    }
}

fn sanitize(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("【.*】").unwrap();
    }
    let s = RE.replace_all(&s, "").trim().to_string();
    s.replace("&amp;", "").replace("&nbsp;", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sanitize_braces() {
        let result = sanitize("最強爆流コンボデッキ60【カメックス＋キュレムEX】");
        let expected = "最強爆流コンボデッキ60".to_string();
        assert_eq!(result, expected);
    }
}
