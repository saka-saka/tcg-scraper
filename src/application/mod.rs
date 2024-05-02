mod one_piece;
pub mod ptcg;
mod ptcg_jp;
mod ws;
mod yugioh;

use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

use self::{one_piece::OnePiece, ptcg::Ptcg, ptcg_jp::PtcgJp, ws::Ws, yugioh::Yugioh};
use crate::{
    repository::Repository,
    scraper::{
        one_piece::OnePieceScraper, pokemon_wiki::PokemonWikiScraper, ptcg::PtcgScraper,
        tcg_collector::TcgCollectorScraper, ws::WsScraper, yugioh::YugiohScraper,
    },
};
use std::{borrow::Cow, io::Write, path::Path};

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

#[derive(Clone)]
pub struct GcsDownloader {
    pub client: google_cloud_storage::client::Client,
    pub bucket: String,
    pub base_path: String,
}

impl GcsDownloader {
    async fn download(&self, url: url::Url) -> Result<(), crate::error::Error> {
        let mut iter = url.path_segments().unwrap().rev();
        let filename = iter.next().unwrap();
        let folder = iter.next().unwrap();
        let resp = reqwest::get(url.clone()).await.unwrap();
        let mut media = Media::new(format!("{}/{folder}/{filename}", self.base_path));
        media.content_type = Cow::from("image/jpeg");
        let _result = self
            .client
            .upload_streamed_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                resp.bytes_stream(),
                &UploadType::Simple(media),
            )
            .await
            .unwrap();
        Ok(())
    }
}

pub struct Application {
    repository: Repository,
}

impl Application {
    pub fn new(url: &str) -> Self {
        let repository = Repository::from_dsn(url).unwrap();
        Self { repository }
    }
    pub fn ptcg(&self) -> Ptcg {
        let scraper = PtcgScraper::new();
        Ptcg {
            repository: self.repository.clone(),
            scraper,
            wiki_scraper: PokemonWikiScraper::new(),
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
