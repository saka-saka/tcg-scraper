pub mod pokemon;

use crate::domain::{LastFetchedAt, PtcgRarity};
use crate::scraper::one_piece::{OnePieceCard, OnePieceCardRarity, OnePieceCardType};
use crate::scraper::pokemon_wiki::PokemonWikiCard;
use crate::scraper::tcg_collector::{
    PtcgJpCard, PtcgJpExpansion, TcgCollectorCardDetail, TcgCollectorCardRarity,
};
use crate::scraper::ws::WsCard;
use crate::scraper::yugioh::YugiohPrinting;
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use pokemon::PokemonRepository;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Transaction};

#[derive(Clone)]
pub struct Repository {
    pool: Pool<Postgres>,
}

impl Repository {
    pub fn pokemon(&self) -> PokemonRepository {
        PokemonRepository {
            pool: self.pool.clone(),
        }
    }
    pub fn from_dsn(url: &str) -> Result<Self, RepositoryError> {
        let pool = PgPoolOptions::new().connect_lazy(url)?;
        Ok(Self { pool })
    }

    pub async fn upsert_pokewiki(
        &self,
        cards: Vec<PokemonWikiCard>,
    ) -> Result<(), RepositoryError> {
        let unzipped = cards
            .into_iter()
            .fold((vec![], vec![], vec![], vec![]), |mut acc, card| {
                acc.0.push(card.number);
                acc.1.push(card.name);
                acc.2.push(card.exp_code);
                acc.3.push(card.rarity);
                acc
            });
        sqlx::query!(
            "
    INSERT INTO pokewiki(number, name, exp_code, rarity)
    SELECT * FROM UNNEST($1::TEXT[], $2::TEXT[], $3::TEXT[], $4::ptcg_rarity_enum[])
    ON CONFLICT (number, name, rarity, exp_code)
    DO NOTHING
    ",
            &unzipped.0,
            &unzipped.1,
            &unzipped.2,
            &unzipped.3 as &Vec<PtcgRarity>,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub fn get_tc_details(&self) -> BoxStream<Result<TcgCollectorCardDetail, RepositoryError>> {
        sqlx::query_as!(
            TcgCollectorCardDetail,
            r#"SELECT name, number, exp_code, html, url, rarity AS "rarity: _" from tcg_collector"#
        )
        .fetch(&self.pool)
        .map_err(RepositoryError::from)
        .boxed()
    }
    pub async fn get_ptcg_jp_expansions_links(&self) -> Result<Vec<String>, RepositoryError> {
        let links = sqlx::query!(
            "
            SELECT exp_link
            FROM pokemon_trainer_expansion pt
            LEFT JOIN ptcg_jp_expansions jp on pt.code = jp.code"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(links.into_iter().filter_map(|r| r.exp_link).collect())
    }
    pub async fn save_tcg_collector(
        &self,
        details: Vec<TcgCollectorCardDetail>,
    ) -> Result<(), RepositoryError> {
        for detail in details {
            sqlx::query!(
                "INSERT INTO tcg_collector(name, number, exp_code, html, url) VALUES ($1, $2, $3, $4, $5)",
                detail.name,
                detail.number,
                detail.exp_code,
                detail.html,
                detail.url
            ).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn update_tc_rarity(
        &self,
        card_rarities: Vec<TcgCollectorCardRarity>,
    ) -> Result<(), RepositoryError> {
        for c in card_rarities {
            sqlx::query!(
                "UPDATE tcg_collector SET rarity = $1 WHERE url = $2",
                c.rarity as PtcgRarity,
                c.url
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
    pub async fn save_ptcg_jp_expansions(
        &self,
        exps: Vec<PtcgJpExpansion>,
    ) -> Result<(), RepositoryError> {
        for exp in exps {
            match sqlx::query!(
                            "INSERT INTO ptcg_jp_expansions(code, name_en, exp_link, symbol_src, logo_src, release_date)
                            VALUES($1, $2, $3, $4, $5, $6)",
                            exp.code,
                            exp.name,
                            exp.link,
                            exp.symbol_src,
                            exp.logo_src,
                            exp.release_date
                        )
                        .execute(&self.pool)
                        .await {
                Ok(_) => {},
                Err(err) => {dbg!(err);},
            };
        }
        Ok(())
    }
    pub async fn save_ptcg_jp_cards(&self, cards: Vec<PtcgJpCard>) -> Result<(), RepositoryError> {
        for card in cards {
            dbg!(&card);
            sqlx::query!(
                "
                UPDATE pokemon_trainer_printing SET
                name_en = $1,
                skill1_name_en = $4,
                skill1_damage = $5,
                card_description_en = $6
                WHERE number = $2 AND expansion_code = $3
                ",
                card.name,
                card.number,
                card.exp_code,
                card.skill1_name_en,
                card.skill1_damage,
                card.desc,
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
    pub async fn upsert_yugioh_expansion_link(&self, url: &str) {
        sqlx::query!(
            "INSERT INTO yugioh_expansion_link(url) VALUES($1) ON CONFLICT(url) DO NOTHING",
            url
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn get_yugioh_expansion_link(&self) -> Option<ExpansionLink> {
        let mut conn = self.pool.begin().await.unwrap();
        let link = sqlx::query!(
            "
            SELECT url FROM yugioh_expansion_link
            LIMIT 1 FOR UPDATE SKIP LOCKED
            "
        )
        .fetch_optional(&mut *conn)
        .await
        .unwrap();
        link.map(|link| ExpansionLink {
            conn,
            url: link.url,
        })
    }
    pub async fn get_yugioh_printing_link(&self) -> Option<PrintingLink> {
        let mut conn = self.pool.begin().await.unwrap();
        let link =
            sqlx::query!("SELECT url FROM yugioh_printing_link LIMIT 1 FOR UPDATE SKIP LOCKED")
                .fetch_optional(&mut *conn)
                .await
                .unwrap();
        link.map(|link| PrintingLink {
            conn,
            url: link.url,
        })
    }
    pub async fn upsert_yugioh_printing_link(&self, url: &str) {
        sqlx::query!(
            "INSERT INTO yugioh_printing_link(url) VALUES($1) ON CONFLICT(url) DO NOTHING",
            url
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn upsert_yugioh_printing_detail(&self, detail: YugiohPrinting) {
        sqlx::query!(
            "
            INSERT INTO yugioh_printing_detail(
            name_jp, name_en, rarity, number, release_date, remark, expansion_name, expansion_code, card_id)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(card_id, expansion_name, rarity)
            DO UPDATE
            SET name_jp = $1, name_en = $2, number = $4, release_date = $5, remark = $6, expansion_code = $7
            ",
            detail.name_jp,
            detail.name_en,
            detail.rarity,
            detail.number,
            detail.release_date,
            detail.remark,
            detail.expansion_name,
            detail.r#ref,
            detail.card_id
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn get_yugioh_printing(&self) -> Result<Vec<YugiohPrinting>, RepositoryError> {
        let printings = sqlx::query!("SELECT * FROM yugioh_printing_detail")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|record| YugiohPrinting {
                card_id: record.card_id,
                name_jp: record.name_jp,
                name_en: record.name_en,
                rarity: record.rarity,
                number: record.number,
                release_date: record.release_date,
                remark: record.remark.unwrap_or(String::from("")),
                expansion_name: record.expansion_name,
                r#ref: record.expansion_code,
            })
            .collect();
        Ok(printings)
    }

    pub async fn upsert_one_piece(&self, card: OnePieceCard) {
        sqlx::query!(
            "
            INSERT INTO one_piece(code, name, img_src, rarity, set_name, type, get_info)
            VALUES($1, $2, $3, $4, $5, $6, $7)",
            card.code,
            card.name,
            card.img_src,
            card.rarity as OnePieceCardRarity,
            card.set_name,
            card.r#type as OnePieceCardType,
            card.get_info,
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub fn list_one_piece(&self) -> BoxStream<Result<OnePieceCard, RepositoryError>> {
        sqlx::query_as!(
            OnePieceCardDto,
            r#"
            SELECT code, name, img_src, rarity AS "rarity!: _", set_name, type AS "type!: _", get_info
            FROM one_piece
            "#,
        )
        .fetch(&self.pool)
        .map_ok(|c|c.into())
        .map_err(|e|e.into())
        .boxed()
    }
    pub async fn get_ws_progress(&self) -> Result<i32, RepositoryError> {
        let record = sqlx::query!(
            "SELECT current_page FROM ws_progress WHERE id = (SELECT id FROM ws_progress_id_seq)"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(record.current_page)
    }
    pub async fn update_ws_progress(&self, current_page: i32) -> Result<(), RepositoryError> {
        sqlx::query!("UPDATE ws_progress SET current_page = $1 WHERE id = (SELECT id FROM ws_progress_id_seq)", current_page)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub async fn save_ws_cards(&self, cards: Vec<WsCard>) -> Result<(), RepositoryError> {
        let unzipped = cards.into_iter().fold(
            (vec![], vec![], vec![], vec![], vec![], vec![]),
            |mut acc, card| {
                acc.0.push(card.code);
                acc.1.push(card.name);
                acc.2.push(card.set_code);
                acc.3.push(card.img_src);
                acc.4.push(card.rarity.unwrap_or("UNKNOWN".to_string()));
                acc.5.push(card.set_name);
                acc
            },
        );
        sqlx::query!(
            "
            INSERT INTO ws_cards(code, name, set_code, img_src, rarity, set_name)
            SELECT *
            FROM UNNEST($1::TEXT[], $2::TEXT[], $3::TEXT[], $4::TEXT[], $5::TEXT[], $6::TEXT[])
            ",
            &unzipped.0,
            &unzipped.1,
            &unzipped.2,
            &unzipped.3,
            &unzipped.4,
            &unzipped.5
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub fn get_ws_cards(&self) -> BoxStream<Result<WsCard, RepositoryError>> {
        sqlx::query_as!(
            WsCardDto,
            "SELECT code, name, set_code, img_src, rarity, set_name FROM ws_cards"
        )
        .fetch(&self.pool)
        .map_ok(|dto| dto.into())
        .map_err(RepositoryError::from)
        .boxed()
    }
}

#[derive(Debug)]
pub struct WsCardDto {
    pub name: String,
    pub code: String,
    pub set_code: String,
    pub img_src: String,
    pub rarity: Option<String>,
    pub set_name: String,
}

impl From<WsCardDto> for WsCard {
    fn from(value: WsCardDto) -> Self {
        Self {
            name: value.name,
            code: value.code,
            set_code: value.set_code,
            img_src: value.img_src,
            rarity: value.rarity,
            set_name: value.set_name,
            last_fetched_at: LastFetchedAt::default(),
        }
    }
}

#[derive(Debug)]
pub struct OnePieceCardDto {
    pub name: String,
    pub code: String,
    pub img_src: String,
    pub rarity: OnePieceCardRarity,
    pub set_name: String,
    pub r#type: OnePieceCardType,
    pub get_info: String,
}

impl From<OnePieceCardDto> for OnePieceCard {
    fn from(value: OnePieceCardDto) -> Self {
        Self {
            name: value.name,
            code: value.code,
            img_src: value.img_src,
            rarity: value.rarity,
            set_name: value.set_name,
            r#type: value.r#type,
            get_info: value.get_info,
            last_fetched_at: LastFetchedAt::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("backend error")]
    BackendError(#[from] sqlx::Error),
}

pub struct ExpansionLink<'a> {
    conn: Transaction<'a, Postgres>,
    pub url: String,
}

impl<'a> ExpansionLink<'a> {
    pub async fn done(mut self) {
        sqlx::query!(
            "DELETE FROM yugioh_expansion_link WHERE url = $1",
            &self.url
        )
        .execute(&mut *self.conn)
        .await
        .unwrap();
        self.conn.commit().await.unwrap();
    }
}
pub struct PrintingLink<'a> {
    conn: Transaction<'a, Postgres>,
    pub url: String,
}

impl<'a> PrintingLink<'a> {
    pub async fn done(mut self) {
        sqlx::query!("DELETE FROM yugioh_printing_link WHERE url = $1", &self.url)
            .execute(&mut *self.conn)
            .await
            .unwrap();
        self.conn.commit().await.unwrap();
    }
}
