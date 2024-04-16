use futures::TryStreamExt;

use crate::{error::Error, ptcg_jp_scraper::PtcgScraper, repository::Repository};

pub struct PtcgJp {
    pub scraper: PtcgScraper,
    pub repository: Repository,
}

impl PtcgJp {
    pub async fn build_exp(&self) -> Result<(), Error> {
        let exps = self.scraper.fetch_tc_exps().await?;
        self.repository.save_ptcg_jp_expansions(exps).await?;
        Ok(())
    }
    pub async fn save_html(&self) -> Result<(), Error> {
        let links = self.repository.get_ptcg_jp_expansions_links().await?;
        for link in links {
            let details = self
                .scraper
                .fetch_tcg_collector_card_detail_html(&link)
                .await?;
            self.repository.save_tcg_collector(details).await?;
        }
        Ok(())
    }
    pub async fn build_cards(&self) -> Result<(), Error> {
        self.repository
            .get_tc_details()
            .map_err(Error::from)
            .try_for_each(|d| async move {
                let card = self.scraper.fetch_card_detail2(d).await?;
                self.repository.save_ptcg_jp_cards(vec![card]).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn build_extra(&self) -> Result<(), Error> {
        self.repository
            .get_tc_details()
            .map_err(Error::from)
            .try_for_each(|d| async move {
                if !self.repository.ptcg_tw_is_exists(&d).await? {
                    let card = self.scraper.fetch_card_detail2(d).await?;
                    dbg!(&card);
                }
                // let card = self.scraper.fetch_card_detail2(d).await?;
                // self.repository.save_ptcg_jp_cards(vec![card]).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
}
