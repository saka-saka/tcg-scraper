use crate::{export_csv::ExportCsv, repository::Repository, yugioh_scraper::YugiohScraper};

pub struct Yugioh {
    pub scraper: YugiohScraper,
    pub repository: Repository,
}

impl Yugioh {
    pub async fn build_yugioh_expansion_link(&self) {
        let expansion_links = self.scraper.fetch_expansion_link().await.unwrap();
        for link in expansion_links {
            self.repository.upsert_yugioh_expansion_link(&link).await;
        }
    }
    pub async fn build_yugioh_printing_link(&self) -> Option<()> {
        let link = self.repository.get_yugioh_expansion_link().await?;
        let url = format!(
            "https://www.db.yugioh-card.com{}&request_locale=ja",
            link.url
        );
        let printing_links = self.scraper.fetch_printing_link(&url).await.unwrap();
        for link in printing_links {
            self.repository.upsert_yugioh_printing_link(&link).await;
        }
        link.done().await;
        Some(())
    }
    pub async fn build_yugioh_printing_detail(&self) -> Option<()> {
        let link = self.repository.get_yugioh_printing_link().await?;
        let url = format!(
            "https://www.db.yugioh-card.com{}&request_locale=ja",
            link.url
        );
        let printings = self.scraper.fetch_printing_detail(&url).await.unwrap();
        for printing in printings {
            self.repository
                .upsert_yugioh_printing_detail(printing)
                .await
        }
        link.done().await;
        Some(())
    }
    pub async fn export_yugioh_printing_detail<W: std::io::Write>(&self, w: W) {
        let mut wtr = csv::Writer::from_writer(w);
        for printing in self.repository.get_yugioh_printing().await.unwrap() {
            let p: ExportCsv = printing.into();
            wtr.serialize(p).unwrap();
        }
        wtr.flush().unwrap();
    }
}
