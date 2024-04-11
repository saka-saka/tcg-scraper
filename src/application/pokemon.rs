use crate::{
    domain::{PokemonCard, Rarity},
    pokemon_trainer_scraper::{PokemonTrainerSiteScraper, ThePTCGSet},
    repository::Repository,
};
use futures::StreamExt;
use std::io::Write;
use strum::IntoEnumIterator;

pub struct PokemonTrainer {
    pub repository: Repository,
    pub scraper: PokemonTrainerSiteScraper,
}

impl PokemonTrainer {
    pub async fn download_all_image(&self) {
        let codes = self.repository.get_ptcg_codes();
        codes
            .for_each(|code| async move {
                let client = reqwest::Client::new();
                let code: i32 = code.parse().unwrap();
                let image_url = format!(
                    "https://asia.pokemon-card.com/tw/card-img/tw{:08}.png",
                    code
                );
                let result = match client.get(&image_url).send().await {
                    Ok(r) => r,
                    Err(_) => return,
                };
                let paths = result.url().path_segments().unwrap();
                let filename = paths.last().unwrap();
                let save_path = format!("images/{}", filename);
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(save_path)
                    .unwrap();
                let bytes = result.bytes().await.unwrap();
                let _ = file.write(&bytes).unwrap();
            })
            .await;
    }
    pub async fn update_entire_pokemon_trainer_expansion(&self) -> Result<(), crate::error::Error> {
        let expansions = self.scraper.fetch_expansion().await?;
        for set in expansions {
            self.repository.upsert_pokemon_trainer_expansion(&set).await;
        }
        Ok(())
    }
    pub async fn build_pokemon_trainer_fetchable(&self) -> Result<(), crate::error::Error> {
        let expansion_codes = self
            .repository
            .get_all_pokemon_trainer_expansion_code()
            .await?;
        for expansion_code in expansion_codes {
            let fetchable_codes = self
                .scraper
                .get_fetchables_by_set(&expansion_code)
                .await
                .unwrap();
            self.repository
                .upsert_fetchable(fetchable_codes, &expansion_code)
                .await;
        }
        Ok(())
    }
    pub async fn update_pokemon_trainer_printing(&self) {
        let fetchable_codes = self.repository.get_fetchable().await;
        for (code, set_code) in fetchable_codes {
            let mut card = match self
                .scraper
                .fetch_printing_detail(&format!(
                    "https://asia.pokemon-card.com/tw/card-search/detail/{code}/"
                ))
                .await
            {
                Ok(card) => card,
                Err(err) => {
                    println!("{err} - {code}");
                    continue;
                }
            };
            card.set_code = Some(set_code);
            self.repository.upsert_the_ptcg_card(&card).await;
            self.repository.fetched(&code).await;
        }
    }
    pub async fn list_all_expansions(&self) -> Result<Vec<ThePTCGSet>, crate::error::Error> {
        let expansions = self.scraper.fetch_expansion().await?;
        Ok(expansions)
    }
    pub async fn update_rarity(&self) {
        for rarity in Rarity::iter() {
            let ids = self.scraper.rarity_ids(&rarity).await.unwrap();
            self.repository.update_the_ptcg_rarity(ids, &rarity).await;
        }
    }
    pub async fn export_pokemon_trainer(&self) -> Result<Vec<PokemonCard>, crate::error::Error> {
        let all_cards = self.repository.get_all_pokemon_trainer_printing();
        Ok(all_cards.collect().await)
    }
}
