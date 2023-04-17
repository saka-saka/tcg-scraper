use crate::bigweb_scraper::BigwebScraper;
use crate::domain::{CardsetURL, PokemonCard};
use crate::repository::BigwebRepository;
use tracing::{debug, error};

pub struct Application {
    bigweb_scraper: BigwebScraper,
    bigweb_repository: BigwebRepository,
}

impl Application {
    pub fn new(url: &str) -> Self {
        let bigweb_scraper = BigwebScraper::new().unwrap();
        let bigweb_repository = BigwebRepository::new(url);
        Self {
            bigweb_scraper,
            bigweb_repository,
        }
    }
    pub async fn update_entire_cardset_db(&self) -> Result<(), Error> {
        let pokemon_cardsets = &self.bigweb_scraper.fetch_pokemon_cardset()?;
        let (sets, errs): (Vec<_>, Vec<_>) =
            pokemon_cardsets
                .iter()
                .fold((vec![], vec![]), |mut acc, elem| {
                    match elem {
                        Ok(result) => acc.0.push(result),
                        Err(err) => acc.1.push(err),
                    };
                    acc
                });
        for err in errs {
            error!(?err)
        }
        for cardset in sets {
            debug!(?cardset);
            self.bigweb_repository.upsert_cardset(cardset).await?;
        }
        Ok(())
    }
    pub async fn update_single_set_card_db(&self, set_ref: &str) -> Result<(), Error> {
        let set_id = self
            .bigweb_repository
            .get_cardset_id(set_ref)
            .await?
            .ok_or(Error::SetNotExists(set_ref.to_string()))?;
        self.update_whole_set_card_db(&set_id).await?;
        Ok(())
    }
    async fn update_whole_set_card_db(&self, set_id: &str) -> Result<(), Error> {
        let cardset_url = CardsetURL::from_cardset_id(set_id).unwrap();
        let cards = self
            .bigweb_scraper
            .fetch_pokemon_data(cardset_url.origin_url().as_str())?;

        let (cards, errs): (Vec<_>, Vec<_>) =
            cards.iter().fold((vec![], vec![]), |mut acc, elem| {
                match elem {
                    Ok(value) => acc.0.push(value),
                    Err(err) => acc.1.push(err),
                };
                acc
            });
        for card in cards {
            self.bigweb_repository.upsert_card(card).await?;
        }
        if !errs.is_empty() {
            for err in errs {
                error!(?err);
            }
        } else {
            self.bigweb_repository.synced(set_id).await?;
        }
        Ok(())
    }
    pub async fn update_entire_card_db(&self) -> Result<(), Error> {
        let cardset_ids = self.bigweb_repository.get_cardset_ids(false).await?;
        for set_id in cardset_ids {
            match self.update_whole_set_card_db(&set_id).await {
                Ok(_) => {}
                Err(err) => {
                    error!(?err)
                }
            };
        }
        Ok(())
    }
    pub async fn export_entire_card_db(&self) -> Result<Vec<PokemonCard>, Error> {
        let all_cards = self.bigweb_repository.fetch_all_cards().await?;
        Ok(all_cards)
    }
    pub async fn unsync_entire_cardset_db(&self) -> Result<(), Error> {
        let all_sets = self.bigweb_repository.get_cardset_ids(true).await?;
        for set_id in all_sets {
            self.bigweb_repository.unsync(&set_id).await?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bigweb scraper error")]
    Scraper(#[from] crate::scraper_error::Error),
    #[error("bigweb repository error")]
    Repository(#[from] crate::repository::Error),
    #[error("set is not exist {0}")]
    SetNotExists(String),
}
