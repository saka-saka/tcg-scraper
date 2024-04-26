use futures::TryStreamExt;
use google_cloud_storage::client::ClientConfig;

use crate::{
    error::Error, export::export_csv::ExportCsv, repository::Repository, scraper::ws::WsScraper,
};

use super::GcsDownloader;

pub struct Ws {
    pub scraper: WsScraper,
    pub repository: Repository,
}

impl Ws {
    pub async fn download_images(&self, bucket: &str, base_path: &str) -> Result<(), Error> {
        let stream = self.repository.get_ws_cards();
        let config = ClientConfig::default().with_auth().await.unwrap();
        let client = google_cloud_storage::client::Client::new(config);
        let downloader = &GcsDownloader {
            client,
            bucket: bucket.to_string(),
            base_path: base_path.to_string(),
        };
        stream
            .map_err(Error::from)
            .try_for_each(|c| async move {
                let image_url = url::Url::parse(&format!("https://ws-tcg.com{}", c.img_src))?;
                downloader.download(image_url).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn scrape(&self) -> Result<(), Error> {
        let total_pages = self.scraper.get_total_pages().await?;
        let progress = self.repository.get_ws_progress().await?;
        for n in progress + 1..=total_pages {
            let cards = self.scraper.scrape_by_page(n).await?;
            let cards = cards.into_iter().filter_map(|s| s.ok()).collect();
            self.repository.save_ws_cards(cards).await?;
            self.repository.update_ws_progress(n + 1).await?;
        }
        Ok(())
    }

    pub async fn export_csv<W: std::io::Write>(&self, w: W) -> Result<(), Error> {
        let mut wtr = csv::Writer::from_writer(w);
        let mut s = self.repository.get_ws_cards();
        while let Some(card) = s.try_next().await? {
            let p: ExportCsv = card.into();
            wtr.serialize(p)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
