use crate::domain::{LastFetchedAt, PokemonCard, PtcgRarity};
use crate::scraper::one_piece::{OnePieceCard, OnePieceCardRarity, OnePieceCardType};
use crate::scraper::pokemon_wiki::PokemonWikiCard;
use crate::scraper::ptcg::{PtcgExpansion, ThePTCGCard};
use crate::scraper::tcg_collector::{
    PtcgJpCard, PtcgJpExpansion, TcgCollectorCardDetail, TcgCollectorCardRarity,
};
use crate::scraper::ws::WsCard;
use crate::scraper::yugioh::YugiohPrinting;
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Transaction};

#[derive(Clone)]
pub struct Repository {
    pool: Pool<Postgres>,
}

impl Repository {
    pub fn from_dsn(url: &str) -> Result<Self, RepositoryError> {
        let pool = PgPoolOptions::new().connect_lazy(url)?;
        Ok(Self { pool })
    }

    pub fn find_ptcg_expansion(&self) -> BoxStream<Result<PtcgExpansion, RepositoryError>> {
        sqlx::query_as!(
            PtcgExpansion,
            "SELECT code, series, name, release_date FROM pokemon_trainer_expansion"
        )
        .fetch(&self.pool)
        .map_err(RepositoryError::from)
        .boxed()
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
    SELECT *
    FROM UNNEST($1::TEXT[], $2::TEXT[], $3::TEXT[], $4::ptcg_rarity_enum[])
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
    pub async fn ptcg_tw_is_exists(
        &self,
        detail: &TcgCollectorCardDetail,
    ) -> Result<bool, RepositoryError> {
        let r = sqlx::query!(
            "SELECT * FROM pokemon_trainer_printing WHERE number = $1 AND expansion_code = $2",
            detail.number,
            detail.exp_code
        )
        .fetch_one(&self.pool)
        .await
        .is_ok();
        Ok(r)
    }
    pub async fn save_extra(&self, card: PtcgJpCard) -> Result<(), RepositoryError> {
        let results = sqlx::query!(
            r#"
            -- INSERT INTO pokemon_trainer_printing(code, name, kind, number, rarity, expansion_code, name_en, skill1_name_en, skill1_damage, card_description_en)
            SELECT
                'd|' || code || '|' || $4 || '|' || $5 as code
            FROM pokemon_trainer_printing
            WHERE name_en = $1 AND (skill1_name_en = $2 OR card_description_en = $3)
            LIMIT 1"#,
            card.name,
            card.skill1_name_en,
            card.desc,
            card.number,
            card.exp_code,
        )
        .fetch_all(&self.pool)
        .await?;
        dbg!(card);
        dbg!(results);
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
    pub async fn upsert_ptcg_expansion(&self, exp: &PtcgExpansion) -> Result<(), RepositoryError> {
        sqlx::query!(
              "INSERT INTO pokemon_trainer_expansion(id, code, series, name, release_date, updated_at)
              VALUES(gen_random_uuid(), $1, $2, $3, $4, NOW())
              ON CONFLICT(code)
              DO UPDATE SET code = $1, series = $2, name = $3, release_date = $4, updated_at = NOW()",
              exp.code, exp.series, exp.name, exp.release_date)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn upsert_fetchable(
        &self,
        fetchable_codes: Vec<String>,
        set_code: &str,
    ) -> Result<(), RepositoryError> {
        for code in fetchable_codes {
            sqlx::query!(
                "INSERT INTO pokemon_trainer_fetchable_card(code, fetched, expansion_code) VALUES($1, False, $2)
                ON CONFLICT(code)
                DO UPDATE
                    SET code = $1, fetched = False, expansion_code = $2",
                code,
                set_code
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
    pub fn get_fetchable(&self) -> BoxStream<Result<(String, String), RepositoryError>> {
        sqlx::query!(
            "SELECT fetchable.code, fetchable.expansion_code
            FROM pokemon_trainer_fetchable_card fetchable
            WHERE fetched = false
            "
        )
        .fetch(&self.pool)
        .map_ok(|s| (s.code, s.expansion_code))
        .map_err(RepositoryError::from)
        .boxed()
    }
    pub fn get_fetchable_by_code(&self, code: &str) -> BoxStream<Result<String, RepositoryError>> {
        sqlx::query!(
            "SELECT code FROM pokemon_trainer_fetchable_card WHERE expansion_code = $1",
            code
        )
        .fetch(&self.pool)
        .map_ok(|s| s.code)
        .map_err(RepositoryError::from)
        .boxed()
    }
    pub async fn upsert_the_ptcg_card(&self, card: &ThePTCGCard) {
        sqlx::query!(
            "
                   INSERT INTO pokemon_trainer_printing(code, kind, name, number, expansion_code)
                   VALUES($1, $2, $3, $4, $5)
                   ON CONFLICT(name, number, expansion_code)
                   DO UPDATE
                   SET kind = $2, name = $3, number = $4, expansion_code = $5
                   ",
            card.code,
            card.kind,
            card.name,
            card.number,
            card.set_code
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn fetched(&self, code: &str) {
        sqlx::query!(
            "UPDATE pokemon_trainer_fetchable_card SET fetched = True WHERE code = $1",
            code
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn update_the_ptcg_rarity(
        &self,
        ids: Vec<String>,
        rarity: &PtcgRarity,
    ) -> Result<(), RepositoryError> {
        dbg!(rarity.to_string());
        sqlx::query!(
            "UPDATE pokemon_trainer_printing SET rarity = $1 WHERE code = ANY($2)",
            rarity.to_string(),
            &ids
        )
        .execute(&self.pool)
        .await?;
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

    pub(crate) fn get_ptcg_codes(&self) -> BoxStream<Result<String, RepositoryError>> {
        sqlx::query!("SELECT code FROM pokemon_trainer_expansion")
            .fetch(&self.pool)
            .map_ok(|c| c.code)
            .map_err(|e| e.into())
            .boxed()
    }

    pub(crate) fn get_all_pokemon_trainer_printing(&self) -> BoxStream<PokemonCard> {
        sqlx::query!(
            "SELECT
            p.code as id,
            p.name as name,
            p.number as number,
            NULL::bigint as sale_price,
            p.rarity as rarity,
            e.code as set_id,
            e.name as set_name,
            e.code as set_ref,
            NULL as remark
            FROM pokemon_trainer_printing p
            LEFT JOIN pokemon_trainer_expansion e ON p.expansion_code = e.code"
        )
        .fetch(&self.pool)
        .filter_map(|r| async {
            let record = r.ok()?;
            Some(PokemonCard {
                id: record.id,
                set_id: record.set_id.unwrap(),
                set_name: record.set_name.unwrap(),
                name: record.name,
                number: Some(record.number),
                set_ref: record.set_ref,
                sale_price: record.sale_price,
                rarity: record.rarity,
                remark: record.remark,
                last_fetched_at: LastFetchedAt::default(),
            })
        })
        .boxed()
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
