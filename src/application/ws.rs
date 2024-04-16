use futures::TryStreamExt;

use crate::{error::Error, export_csv::ExportCsv, repository::Repository, ws_scraper::WsScraper};

pub struct Ws {
    pub scraper: WsScraper,
    pub repository: Repository,
}

impl Ws {
    pub async fn ws_scrape(&self) -> Result<(), Error> {
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

    pub async fn ws_export_csv<W: std::io::Write>(&self, w: W) -> Result<(), Error> {
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
