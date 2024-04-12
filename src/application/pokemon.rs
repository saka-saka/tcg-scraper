use crate::{
    domain::{PokemonCard, Rarity},
    error::Error,
    pokemon_trainer_scraper::{PokemonTrainerSiteScraper, ThePTCGSet},
    repository::Repository,
};
use futures::{StreamExt, TryStreamExt};
use strum::IntoEnumIterator;
use url::Url;

use super::download;

pub struct PokemonTrainer {
    pub repository: Repository,
    pub scraper: PokemonTrainerSiteScraper,
}

impl PokemonTrainer {
    pub async fn download_all_image(&self) -> Result<(), Error> {
        let codes = self.repository.get_ptcg_codes();
        codes
            .map_err(Error::from)
            .try_for_each(|code| async move {
                let code: i32 = code.parse().unwrap();
                let image_url = format!(
                    "https://asia.pokemon-card.com/tw/card-img/tw{:08}.png",
                    code
                );
                let image_url = Url::parse(&image_url)?;
                download(image_url, "./images/").await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn update_entire_pokemon_trainer_expansion(&self) -> Result<(), Error> {
        let expansions = self.scraper.fetch_expansion().await?;
        for set in expansions {
            self.repository.upsert_pokemon_trainer_expansion(&set).await;
        }
        Ok(())
    }
    pub async fn build_pokemon_trainer_fetchable(&self) -> Result<(), Error> {
        let expansion_codes = self
            .repository
            .get_all_pokemon_trainer_expansion_code()
            .await?;
        for expansion_code in expansion_codes {
            let fetchable_codes = self.scraper.get_fetchables_by_set(&expansion_code).await?;
            self.repository
                .upsert_fetchable(fetchable_codes, &expansion_code)
                .await?;
        }
        Ok(())
    }
    pub async fn update_pokemon_trainer_printing(&self) -> Result<(), Error> {
        self.repository
            .get_fetchable()
            .map_err(Error::from)
            .try_for_each(|(code, set_code)| async move {
                let mut card = self
                    .scraper
                    .fetch_printing_detail(&format!(
                        "https://asia.pokemon-card.com/tw/card-search/detail/{code}/"
                    ))
                    .await?;
                card.set_code = Some(set_code);
                self.repository.upsert_the_ptcg_card(&card).await;
                self.repository.fetched(&code).await;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn list_all_expansions(&self) -> Result<Vec<ThePTCGSet>, Error> {
        let expansions = self.scraper.fetch_expansion().await?;
        Ok(expansions)
    }
    pub async fn update_rarity(&self) -> Result<(), Error> {
        for rarity in Rarity::iter() {
            let ids = self.scraper.rarity_ids(&rarity).await?;
            self.repository.update_the_ptcg_rarity(ids, &rarity).await?;
        }
        Ok(())
    }
    pub async fn export_pokemon_trainer(&self) -> Result<Vec<PokemonCard>, Error> {
        let all_cards = self.repository.get_all_pokemon_trainer_printing();
        Ok(all_cards.collect().await)
    }
}
