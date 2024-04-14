use crate::{
    error::Error, export_csv::ExportCsv, one_piece_csv::OnePieceProductsCsv,
    one_piece_scraper::OnePieceScraper, repository::Repository,
};
use futures::TryStreamExt;

use super::download;

pub struct OnePiece {
    pub scraper: OnePieceScraper,
    pub repository: Repository,
}

impl OnePiece {
    pub async fn download_images(&self) -> Result<(), Error> {
        self.repository
            .list_one_piece()
            .map_err(Error::from)
            .try_for_each(|card| async move {
                let u = url::Url::parse(&card.img_src.clone())?;
                download(u, "./images/").await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn scrape_one_piece(&self) -> Result<(), Error> {
        let sets = self.scraper.set().await?;
        for set in sets {
            for card in self.scraper.scrape_cards(&set).await? {
                self.repository.upsert_one_piece(card.unwrap()).await;
            }
        }
        Ok(())
    }
    pub async fn scrape_one_piece_products(&self) {
        let products = self.scraper.products().await;
        println!("{:#?}", products);
    }
    pub async fn export_one_piece_product_csv<W: std::io::Write>(&self, w: W) -> Result<(), Error> {
        let products = self.scraper.products().await?;
        let mut wtr = csv::Writer::from_writer(w);
        for product in products {
            let c: OnePieceProductsCsv = product.into();
            wtr.serialize(c).unwrap();
        }
        wtr.flush().unwrap();
        Ok(())
    }
    pub async fn export_one_piece_csv<W: std::io::Write>(&self, w: W) -> Result<(), Error> {
        let sets = self.scraper.set().await?;
        let mut wtr = csv::Writer::from_writer(w);
        for set in sets {
            for card in self.scraper.scrape_cards(&set).await? {
                let c: ExportCsv = card.unwrap().into();
                wtr.serialize(c).unwrap();
            }
        }
        wtr.flush().unwrap();
        Ok(())
    }
}
