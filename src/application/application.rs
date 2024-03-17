use super::error::Error;
use super::one_piece::OnePiece;
use super::yugioh::Yugioh;
use crate::bigweb_scraper::BigwebScraper;
use crate::domain::{CardsetURL, PokemonCard};
use crate::limitless_scraper::LimitlessScraper;
use crate::repository::Repository;
use crate::ws_csv::WsCSV;
use crate::ws_scraper::WsScraper;
use crate::yugioh_scraper::YugiohScraper;
use crate::{
    one_piece_scraper::OnePieceScraper, pokemon_trainer_scraper::PokemonTrainerSiteScraper,
};
use std::io::Write;
use tracing::{debug, error};

use super::pokemon::PokemonTrainer;

pub struct Application {
    bigweb_scraper: BigwebScraper,
    repository: Repository,
    ws_scraper: WsScraper,
    limitless_scraper: LimitlessScraper,
}

impl Application {
    pub fn new(url: &str) -> Self {
        let bigweb_scraper = BigwebScraper::new().unwrap();
        let repository = Repository::new(url);
        let ws_scraper = WsScraper {};
        let limitless_scraper = LimitlessScraper::new();
        Self {
            bigweb_scraper,
            repository,
            ws_scraper,
            limitless_scraper,
        }
    }
    pub fn pokemon_trainer(&self) -> PokemonTrainer {
        let scraper = PokemonTrainerSiteScraper::new();
        PokemonTrainer {
            repository: self.repository.clone(),
            scraper,
        }
    }
    pub fn one_piece(&self) -> OnePiece {
        let scraper = OnePieceScraper {};
        OnePiece { scraper }
    }
    pub fn yugioh(&self) -> Yugioh {
        let scraper = YugiohScraper::new();
        Yugioh {
            scraper,
            repository: self.repository.clone(),
        }
    }
    pub async fn scrape_ws(&self) {
        for result in &self.ws_scraper.scrape().await {
            println!("{result:#?}");
        }
    }
    pub async fn download_image(&self) -> Result<(), Error> {
        let all_cards = self.repository.fetch_card_ids().await.unwrap();
        let client = reqwest::Client::new();
        for card in all_cards {
            let image_url = self.bigweb_scraper.fetch_pokemon_card_image(&card).unwrap();
            let result = match client.get(&image_url).send().await {
                Ok(r) => r,
                Err(_) => continue,
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
            let _ = file.write(&bytes)?;
            self.repository.image_downloaded(&card).await.unwrap();
        }
        Ok(())
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
            self.repository.upsert_cardset(cardset).await?;
        }
        Ok(())
    }
    pub async fn update_single_set_card_db(&self, set_ref: &str) -> Result<(), Error> {
        let set_id = self
            .repository
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
            self.repository.upsert_card(card).await?;
        }
        if !errs.is_empty() {
            for err in errs {
                error!(?err);
            }
        } else {
            self.repository.synced(set_id).await?;
        }
        Ok(())
    }
    pub async fn update_entire_card_db(&self) -> Result<(), Error> {
        let cardset_ids = self.repository.get_cardset_ids(false).await?;
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
        let all_cards = self.repository.fetch_all_cards().await?;
        Ok(all_cards)
    }
    pub async fn unsync_entire_cardset_db(&self) -> Result<(), Error> {
        let all_sets = self.repository.get_cardset_ids(true).await?;
        for set_id in all_sets {
            self.repository.unsync(&set_id).await?;
        }
        Ok(())
    }
    pub async fn export_ws_csv<W: std::io::Write>(&self, w: W) {
        let mut wtr = csv::Writer::from_writer(w);
        for card in self.ws_scraper.scrape().await {
            let c: WsCSV = card.unwrap().into();
            wtr.serialize(c).unwrap();
        }
        // for printing in self.repository.get_yugioh_printing().await.unwrap() {
        //     let p: YugiohCsv = printing.into();
        //     wtr.serialize(p).unwrap();
        // }
        wtr.flush().unwrap();
    }
    pub async fn poc(&self) {
        self.limitless_scraper.poc().await;
    }
}
