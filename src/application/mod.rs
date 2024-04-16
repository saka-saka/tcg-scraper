mod one_piece;
mod pokemon;
mod ptcg_jp;
mod ws;
mod yugioh;
use self::{one_piece::OnePiece, pokemon::PokemonTrainer, ptcg_jp::PtcgJp, ws::Ws, yugioh::Yugioh};
use crate::{
    limitless_scraper::LimitlessScraper, one_piece_scraper::OnePieceScraper,
    pokemon_trainer_scraper::PokemonTrainerSiteScraper, ptcg_jp_scraper::PtcgScraper,
    repository::Repository, ws_scraper::WsScraper, yugioh_scraper::YugiohScraper,
};
use std::{io::Write, path::Path};

async fn download<T: AsRef<Path>>(url: url::Url, save_path: T) -> Result<(), crate::error::Error> {
    let result = reqwest::get(url).await?;
    let paths = result.url().path_segments().unwrap();
    let file_name = paths.last().unwrap();
    let save_path = save_path.as_ref().join(file_name);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(save_path)?;
    let bytes = result.bytes().await?;
    let _ = file.write(&bytes)?;
    Ok(())
}

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
    pub fn ptcg_jp(&self) -> PtcgJp {
        let scraper = PtcgScraper {};
        PtcgJp {
            scraper,
            repository: self.repository.clone(),
        }
    }
}
