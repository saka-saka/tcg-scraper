use crate::domain::{BigwebScrappedPokemonCard, Cardset, PokemonCard, Rarity};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub struct BigwebRepository {
    pool: Pool<Postgres>,
}

impl BigwebRepository {
    pub fn new(url: &str) -> Self {
        let pool = PgPoolOptions::new().connect_lazy(url).unwrap();
        Self { pool }
    }
    pub async fn get_cardset_id(&self, set_ref: &str) -> Result<Option<String>, Error> {
        let record = sqlx::query!(
            "SELECT id FROM bigweb_pokemon_cardset WHERE ref = $1",
            set_ref
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(record.map(|r| r.id))
    }
    pub async fn get_cardset_ids(&self, is_sync: bool) -> Result<Vec<String>, Error> {
        let cardset_ids: Vec<String> = sqlx::query!(
            "SELECT id FROM bigweb_pokemon_cardset WHERE is_sync = $1",
            is_sync
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|c| c.id.to_owned())
        .collect();
        Ok(cardset_ids)
    }
    pub async fn upsert_cardset(&self, cardset: &Cardset) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO bigweb_pokemon_cardset(id, cardset_name, ref, updated_at, item_count)
            VALUES($1, $2, $3, NOW(), $4)
            ON CONFLICT(id)
            DO UPDATE SET cardset_name = $2, ref = $3, updated_at = NOW(), item_count = $4",
            cardset.url.cardset_id(),
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
            "SELECT bpc.id, name, number, rarity, sale_price, bpcs.cardset_name, bpcs.ref as cardset_ref, set_id, last_fetched_at, remark
            FROM bigweb_pokemon_card bpc
            LEFT JOIN bigweb_pokemon_cardset bpcs
            ON bpc.set_id = bpcs.id"
        )
        .fetch_all(&self.pool)
        .await?;
        let mut pokemon_data = vec![];
        for r in record {
            pokemon_data.push(PokemonCard {
                name: r.name,
                number: r.number,
                id: r.id,
                rarity: r.rarity,
                set_id: r.set_id,
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
    pub async fn upsert_card(&self, card: &BigwebScrappedPokemonCard) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO bigweb_pokemon_card(id, name, number, rarity, sale_price, set_id, remark)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(id)
            DO UPDATE SET name = $2, number = $3, rarity = $4, sale_price = $5, set_id = $6, remark = $7",
            card.id,
            card.name,
            card.number,
            card.rarity.clone().map(|r| {
              match r {
                Rarity::Unknown(s)=> s,
                _ => r.to_string()
              }
            }),
            card.sale_price.clone().map(|p| p.value() as i32),
            card.set_id,
            card.remark
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn synced(&self, cardset_id: &str) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE bigweb_pokemon_cardset
            SET is_sync = true, updated_at = NOW()
            WHERE id = $1",
            cardset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn unsync(&self, cardset_id: &str) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE bigweb_pokemon_cardset
            SET is_sync = false, updated_at = NOW()
            WHERE id = $1",
            cardset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("backend error")]
    BackendError(#[from] sqlx::Error),
}
