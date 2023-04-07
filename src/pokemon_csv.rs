use crate::domain::PokemonCard;
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
            set: Some(value.set_name),
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
