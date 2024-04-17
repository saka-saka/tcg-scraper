use derive_builder::Builder;
use serde::Deserialize;
use sqlx::types::time::OffsetDateTime;
use strum_macros::EnumString;
use time::macros::format_description;

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

#[allow(clippy::upper_case_acronyms)]
#[derive(
    Deserialize,
    EnumString,
    strum_macros::Display,
    Clone,
    Debug,
    PartialEq,
    Default,
    strum_macros::EnumIter,
    sqlx::Type,
)]
#[sqlx(type_name = "ptcg_rarity_enum")]
pub enum Rarity {
    #[default]
    UR,
    SSR,
    ACE,
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
    Unknown,
}
