use super::one_piece::OnePiece;
use super::ws::Ws;
use super::yugioh::Yugioh;
use crate::limitless_scraper::LimitlessScraper;
use crate::repository::Repository;
use crate::ws_scraper::WsScraper;
use crate::yugioh_scraper::YugiohScraper;
use crate::{
    one_piece_scraper::OnePieceScraper, pokemon_trainer_scraper::PokemonTrainerSiteScraper,
};

use super::pokemon::PokemonTrainer;

pub struct Application {
    repository: Repository,
    limitless_scraper: LimitlessScraper,
}

impl Application {
    pub fn new(url: &str) -> Self {
        let repository = Repository::new(url);
        let limitless_scraper = LimitlessScraper::new();
        Self {
            repository,
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
        OnePiece {
            scraper,
            repository: self.repository.clone(),
        }
    }
    pub fn yugioh(&self) -> Yugioh {
        let scraper = YugiohScraper::new();
        Yugioh {
            scraper,
            repository: self.repository.clone(),
        }
    }
    pub fn ws(&self) -> Ws {
        let scraper = WsScraper {};
        Ws {
            scraper,
            repository: self.repository.clone(),
        }
    }
    pub async fn poc(&self) {
        self.limitless_scraper.poc().await;
    }
}
