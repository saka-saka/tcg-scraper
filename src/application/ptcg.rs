use crate::{
    domain::{PokemonCard, PtcgRarity},
    error::Error,
    repository::Repository,
    scraper::{pokemon_wiki::PokemonWikiScraper, ptcg::PtcgScraper},
    strategy::{ManualStrategy, PtcgStrategy, Source, TcgCollectorStrategy, WikiStrategy},
    PtcgExpansionDbRow,
};
use futures::{StreamExt, TryStreamExt};
use strum::IntoEnumIterator;
use url::Url;

use super::download;

#[derive(Clone)]
pub struct Ptcg {
    pub repository: Repository,
    pub scraper: PtcgScraper,
    pub wiki_scraper: PokemonWikiScraper,
}

impl Ptcg {
    pub async fn expansion(
        &self,
        sources: Vec<Source>,
        record: PtcgExpansionDbRow,
    ) -> Result<(), Error> {
        for source in sources {
            match source {
                Source::Ptcg(PtcgStrategy::All) => {
                    // let count = self
                    //     .repository
                    //     .get_fetchable_by_code(&record.exp)
                    //     .count()
                    //     .await;
                    // if count == 0 {
                    //     let fetchable_codes =
                    //         self.scraper.get_fetchables_by_exp(&record.exp).await?;
                    //     self.repository
                    //         .upsert_fetchable(fetchable_codes, &record.exp)
                    //         .await?;
                    // }
                    // let mut card = self
                    //     .scraper
                    //     .fetch_printing_detail(&format!(
                    //         "https://asia.pokemon-card.com/tw/card-search/detail/{}/",
                    //         record.exp
                    //     ))
                    //     .await?;
                    // card.set_code = Some(record.exp.clone());
                    // self.repository.upsert_the_ptcg_card(&card).await;
                    // self.repository.fetched(&record.exp).await;
                }
                Source::Ptcg(PtcgStrategy::Pic) => {}
                Source::Wiki(WikiStrategy::Data(data)) => {
                    let cards = self
                        .wiki_scraper
                        .fetch_card_data_by_exp_url(data.url().as_str(), &record.exp)
                        .await?;
                    self.repository.upsert_pokewiki(cards).await?;
                }
                Source::TcgCollector(TcgCollectorStrategy::Pic(_data)) => {}
                Source::TcgCollector(TcgCollectorStrategy::PicByName(_data)) => {}
                Source::TcgCollector(TcgCollectorStrategy::PicMappings(_data)) => {}
                Source::Manual(ManualStrategy::Data(_card_data)) => {}
            }
        }
        Ok(())
    }
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
    pub async fn prepare_ptcg_expansions(&self) -> Result<(), Error> {
        let count = self.repository.find_ptcg_expansion().count().await;
        if count == 0 {
            let expansions = self.scraper.fetch_expansion().await?;
            for exp in expansions {
                self.repository.upsert_ptcg_expansion(&exp).await?;
            }
        }
        Ok(())
    }
    pub async fn update_ptcg_fetchable(&self) -> Result<(), Error> {
        let codes = self.repository.get_ptcg_codes();
        codes
            .map_err(Error::from)
            .try_for_each(|code| async move {
                let count = self.repository.get_fetchable_by_code(&code).count().await;
                if count == 0 {
                    let fetchable_codes = self.scraper.get_fetchables_by_exp(&code).await?;
                    self.repository
                        .upsert_fetchable(fetchable_codes, &code)
                        .await?;
                }
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn update_ptcg_printing(&self) -> Result<(), Error> {
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
    pub async fn update_rarity(&self) -> Result<(), Error> {
        for rarity in PtcgRarity::iter() {
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
