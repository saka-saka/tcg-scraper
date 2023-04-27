use crate::domain::PokemonCard;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PokemonCSV {
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

impl From<PokemonCard> for PokemonCSV {
    fn from(value: PokemonCard) -> Self {
        PokemonCSV {
            product_id: None,
            brand: Some(String::from("Pokemon")),
            set: Some(sanitize(&value.set_name)),
            edition: None,
            series: None,
            rarity: value.rarity,
            material: None,
            release_year: None,
            language: Some(String::from("ja")),
            card_name_english: None,
            card_name_chinese: None,
            card_name_japanese: Some(value.name),
            card_number: value.number,
            image: None,
            value: value.sale_price.map(|p| p.to_string()),
            reference: Some(value.set_ref),
            remark: None,
            remark1: value.last_fetched_at.action_code(),
            remark2: value.last_fetched_at.created_datetime(),
            remark3: Some(value.set_id),
            remark4: Some(value.id),
            remark5: None,
            remark6: None,
            remark7: None,
            remark8: None,
            remark9: value.remark,
            remark10: None,
            enable: None,
            p_language: None,
            id: None,
        }
    }
}

fn sanitize(s: &str) -> String {
    let s1 = s.replace("【", "").replace("】", "").replace("&amp;", "&");
    lazy_static! {
        static ref RE: Regex = Regex::new("[1-9]種").unwrap();
    }
    RE.replace_all(&s1, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sanitize_x_kind() {
        let result = sanitize("バトル強化デッキ 3種");
        let expected = "バトル強化デッキ".to_string();
        assert_eq!(result, expected);
    }
    #[test]
    fn sanitize_braces() {
        let result = sanitize("最強爆流コンボデッキ60【カメックス＋キュレムEX】");
        let expected = "最強爆流コンボデッキ60カメックス＋キュレムEX".to_string();
        assert_eq!(result, expected);
    }
}
