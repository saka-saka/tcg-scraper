use crate::domain::{LastFetchedAt, PokemonCard, PtcgRarity};
use crate::scraper::ptcg::{PtcgExpansion, ThePTCGCard};
use crate::scraper::tcg_collector::{PtcgJpCard, TcgCollectorCardDetail};
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use sqlx::{Pool, Postgres};

use super::RepositoryError;

pub struct PokemonRepository {
    pub(crate) pool: Pool<Postgres>,
}
impl PokemonRepository {
    pub(crate) fn get_ptcg_exp_codes(&self) -> BoxStream<Result<String, RepositoryError>> {
        sqlx::query!("SELECT code FROM pokemon_trainer_expansion")
            .fetch(&self.pool)
            .map_ok(|c| c.code)
            .map_err(|e| e.into())
            .boxed()
    }

    pub(crate) fn get_ptcg_card_codes(&self) -> BoxStream<Result<String, RepositoryError>> {
        sqlx::query!("SELECT code FROM pokemon_trainer_printing")
            .fetch(&self.pool)
            .map_ok(|c| c.code)
            .map_err(|e| e.into())
            .boxed()
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
    pub fn find_ptcg_expansion(&self) -> BoxStream<Result<PtcgExpansion, RepositoryError>> {
        sqlx::query_as!(
            PtcgExpansion,
            "SELECT code, series, name, release_date FROM pokemon_trainer_expansion"
        )
        .fetch(&self.pool)
        .map_err(RepositoryError::from)
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
}
