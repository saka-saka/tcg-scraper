use crate::domain::{BigwebScrappedPokemonCard, Cardset, LastFetchedAt, PokemonCard, Rarity};
use crate::pokemon_trainer_scraper::{ThePTCGCard, ThePTCGSet};
use crate::yugioh_scraper::YugiohPrinting;
use futures::stream::BoxStream;
use futures::StreamExt;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Transaction};

#[derive(Clone)]
pub struct Repository {
    pool: Pool<Postgres>,
}

impl Repository {
    pub fn new(url: &str) -> Self {
        let pool = PgPoolOptions::new().connect_lazy(url).unwrap();
        Self { pool }
    }
    pub async fn get_cardset_id(&self, set_ref: &str) -> Result<Option<String>, Error> {
        let record = sqlx::query!(
            "SELECT id FROM bigweb_pokemon_expansion WHERE code = $1",
            set_ref
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(record.map(|r| r.id.to_string()))
    }
    pub async fn get_cardset_ids(&self, is_sync: bool) -> Result<Vec<String>, Error> {
        let cardset_ids: Vec<String> = sqlx::query!(
            "SELECT id FROM bigweb_pokemon_expansion WHERE is_sync = $1",
            is_sync
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|c| c.id.to_string())
        .collect();
        Ok(cardset_ids)
    }
    pub async fn upsert_cardset(&self, cardset: &Cardset) -> Result<(), Error> {
        let cardset_id = cardset.url.cardset_id();
        let id = uuid::Uuid::from_slice(cardset_id.as_bytes()).unwrap();
        sqlx::query!(
            "INSERT INTO bigweb_pokemon_expansion(id, name, code, updated_at, item_count)
            VALUES($1, $2, $3, NOW(), $4)
            ON CONFLICT(id)
            DO UPDATE SET name = $2, code = $3, updated_at = NOW(), item_count = $4",
            id,
            cardset.name,
            cardset.r#ref,
            cardset.result_count as i32
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn fetch_all_cards(&self) -> Result<Vec<PokemonCard>, Error> {
        let record = sqlx::query!(
            "SELECT
                bpc.id,
                bpc.name,
                number,
                rarity,
                sale_price,
                bpcs.name cardset_name,
                bpcs.code as cardset_ref,
                expansion_id,
                last_fetched_at,
                remark
            FROM bigweb_pokemon_printing bpc
            LEFT JOIN bigweb_pokemon_expansion bpcs
            ON bpc.expansion_id = bpcs.id"
        )
        .fetch_all(&self.pool)
        .await?;
        let mut pokemon_data = vec![];
        for r in record {
            pokemon_data.push(PokemonCard {
                name: r.name,
                number: r.number,
                id: r.id.to_string(),
                rarity: r.rarity,
                set_id: r.expansion_id.to_string(),
                set_name: r.cardset_name.unwrap(),
                set_ref: r.cardset_ref.unwrap(),
                sale_price: r.sale_price.map(|sp| sp as i64),
                last_fetched_at: crate::domain::LastFetchedAt {
                    inner: r.last_fetched_at.unwrap(),
                },
                remark: r.remark,
            });
        }
        Ok(pokemon_data)
    }
    // pub async fn get_all_pokemon_trainer_printing(&self) -> Result<Vec<PokemonCard>, Error> {
    //     let records = sqlx::query!(
    //         "SELECT printing.code code, printing.name card_name, number, rarity, expansion.name set_name, expansion_code
    //         FROM pokemon_trainer_printing printing
    //         LEFT JOIN pokemon_trainer_expansion expansion
    //         ON printing.expansion_code = expansion.code"
    //     )
    //     .fetch_all(&self.pool)
    //     .await?;
    //     let mut cards = vec![];
    //     for record in records {
    //         let card = PokemonCard {
    //             id: record.code,
    //             set_id: record.set_code,
    //         };
    //         cards.push(card);
    //     }
    //     unimplemented!()
    // }
    pub async fn fetch_card_ids(&self) -> Result<Vec<String>, Error> {
        let record = sqlx::query!(
            "SELECT id
            FROM bigweb_pokemon_printing
            WHERE image_downloaded = false"
        )
        .fetch_all(&self.pool)
        .await?;
        let mut ids = vec![];
        for r in record {
            ids.push(r.id.to_string());
        }
        Ok(ids)
    }
    pub async fn image_downloaded(&self, id: &str) -> Result<(), Error> {
        let id = uuid::Uuid::from_slice(id.as_bytes()).unwrap();
        sqlx::query!(
            "UPDATE bigweb_pokemon_printing SET image_downloaded = true WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn upsert_card(&self, card: &BigwebScrappedPokemonCard) -> Result<(), Error> {
        let card_id = uuid::Uuid::from_slice(card.id.as_bytes()).unwrap();
        let set_id = uuid::Uuid::from_slice(card.set_id.as_bytes()).unwrap();
        sqlx::query!(
            "INSERT INTO bigweb_pokemon_printing(id, name, number, rarity, sale_price, expansion_id, remark)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(id)
            DO UPDATE SET name = $2, number = $3, rarity = $4, sale_price = $5, expansion_id = $6, remark = $7",
            card_id,
            card.name,
            card.number,
            card.rarity.clone().map(|r| {
              match r {
                Rarity::Unknown(s)=> s,
                _ => r.to_string()
              }
            }),
            card.sale_price.clone().map(|p| p.value() as i32),
            set_id,
            card.remark
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn synced(&self, cardset_id: &str) -> Result<(), Error> {
        let cardset_id = uuid::Uuid::from_slice(cardset_id.as_bytes()).unwrap();
        sqlx::query!(
            "UPDATE bigweb_pokemon_expansion
            SET is_sync = true, updated_at = NOW()
            WHERE id = $1",
            cardset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn unsync(&self, cardset_id: &str) -> Result<(), Error> {
        let cardset_id = uuid::Uuid::from_slice(cardset_id.as_bytes()).unwrap();
        sqlx::query!(
            "UPDATE bigweb_pokemon_expansion
            SET is_sync = false, updated_at = NOW()
            WHERE id = $1",
            cardset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn upsert_pokemon_trainer_expansion(&self, set: &ThePTCGSet) {
        sqlx::query!(
              "INSERT INTO pokemon_trainer_expansion(id, code, series, name, release_date, updated_at)
              VALUES(gen_random_uuid(), $1, $2, $3, $4, NOW())
              ON CONFLICT(code)
              DO UPDATE SET code = $1, series = $2, name = $3, release_date = $4, updated_at = NOW()",
              set.expansion_code, set.series, set.name, set.release_date)
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn get_all_pokemon_trainer_expansion_code(&self) -> Result<Vec<String>, Error> {
        let result = sqlx::query!("SELECT code FROM pokemon_trainer_expansion")
            .fetch_all(&self.pool)
            .await?;
        Ok(result.into_iter().map(|a| a.code).collect())
    }
    pub async fn upsert_fetchable(&self, fetchable_codes: Vec<String>, set_code: &str) {
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
            .await
            .unwrap();
        }
    }
    pub async fn get_fetchable(&self) -> Vec<(String, String)> {
        let fetchables = sqlx::query!(
            "SELECT fetchable.code, fetchable.expansion_code
            FROM pokemon_trainer_fetchable_card fetchable
            LEFT JOIN pokemon_trainer_printing printing ON fetchable.code = printing.code
            WHERE printing.name is NULL
            LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();
        fetchables
            .into_iter()
            .map(|s| (s.code, s.expansion_code))
            .collect()
    }
    pub async fn upsert_the_ptcg_card(&self, card: &ThePTCGCard) {
        sqlx::query!(
            "
                   INSERT INTO pokemon_trainer_printing(code, kind, name, number, expansion_code)
                   VALUES($1, $2, $3, $4, $5)
                   ON CONFLICT(code)
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
    pub async fn update_the_ptcg_rarity(&self, ids: Vec<String>, rarity: &Rarity) {
        for id in ids {
            sqlx::query!(
                "UPDATE pokemon_trainer_printing SET rarity = $1 WHERE code = $2",
                rarity.to_string(),
                id
            )
            .execute(&self.pool)
            .await
            .unwrap();
        }
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
        .fetch_optional(&mut conn)
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
                .fetch_optional(&mut conn)
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
    pub async fn get_yugioh_printing(&self) -> Result<Vec<YugiohPrinting>, Error> {
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

    pub(crate) fn get_ptcg_codes(&self) -> BoxStream<String> {
        sqlx::query!("SELECT code FROM pokemon_trainer_printing")
            .fetch(&self.pool)
            .filter_map(|r| async { Some(r.ok()?.code) })
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
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
        .execute(&mut self.conn)
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
            .execute(&mut self.conn)
            .await
            .unwrap();
        self.conn.commit().await.unwrap();
    }
}
