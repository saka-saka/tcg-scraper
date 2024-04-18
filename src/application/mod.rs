mod one_piece;
mod ptcg;
mod ptcg_jp;
mod ws;
mod yugioh;

use self::{one_piece::OnePiece, ptcg::Ptcg, ptcg_jp::PtcgJp, ws::Ws, yugioh::Yugioh};
use crate::{
    repository::Repository,
    scraper::{
        one_piece::OnePieceScraper, ptcg::PtcgScraper, tcg_collector::TcgCollectorScraper,
        ws::WsScraper, yugioh::YugiohScraper,
    },
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
}

impl Application {
    pub fn new(url: &str) -> Self {
        let repository = Repository::new(url);
        Self { repository }
    }
    pub fn pokemon_trainer(&self) -> Ptcg {
        let scraper = PtcgScraper::new();
        Ptcg {
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
    pub fn ptcg_jp(&self) -> PtcgJp {
        let scraper = TcgCollectorScraper {};
        PtcgJp {
            scraper,
            repository: self.repository.clone(),
        }
    }
}
