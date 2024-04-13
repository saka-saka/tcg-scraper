use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExportCsv {
    pub product_id: Option<String>,
    pub brand: Option<String>,
    pub set: Option<String>,
    pub edition: Option<String>,
    pub series: Option<String>,
    pub rarity: Option<String>,
    pub material: Option<String>,
    pub release_year: Option<String>,
    pub language: Option<String>,
    pub card_name_english: Option<String>,
    pub card_name_chinese: Option<String>,
    pub card_name_japanese: Option<String>,
    pub card_number: Option<String>,
    pub image: Option<String>,
    pub value: Option<String>,
    pub reference: Option<String>,
    pub remark: Option<String>,
    pub remark1: Option<String>,
    pub remark2: Option<String>,
    pub remark3: Option<String>,
    pub remark4: Option<String>,
    pub remark5: Option<String>,
    pub remark6: Option<String>,
    pub remark7: Option<String>,
    pub remark8: Option<String>,
    pub remark9: Option<String>,
    pub remark10: Option<String>,
    pub enable: Option<String>,
    #[serde(rename(serialize = "P_Language"))]
    pub p_language: Option<String>,
    #[serde(rename(serialize = "id"))]
    pub id: Option<String>,
}
