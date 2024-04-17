use crate::{export::export_csv::ExportCsv, scraper::ws::WsCard};
use lazy_static::lazy_static;
use regex::Regex;

impl From<WsCard> for ExportCsv {
    fn from(value: WsCard) -> Self {
        ExportCsv {
            product_id: None,
            brand: Some(String::from("Weiβ Schwarz")),
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
            card_number: Some(value.code.clone()),
            image: Some(value.img_src),
            value: None,
            reference: Some(value.set_code.clone()),
            remark: None,
            remark1: value.last_fetched_at.action_code(),
            remark2: value.last_fetched_at.created_datetime(),
            remark3: None,
            remark4: Some(value.code),
            remark5: None,
            remark6: None,
            remark7: None,
            remark8: None,
            remark9: Some(value.set_code),
            remark10: None,
            enable: None,
            p_language: None,
            id: None,
        }
    }
}

fn sanitize(s: &str) -> String {
    let s1 = s.replace(['【', '】'], "").replace("&amp;", "&");
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
