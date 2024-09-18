use futures::TryStreamExt;

use crate::{error::Error, repository::Repository, scraper::tcg_collector::TcgCollectorScraper};

pub struct PtcgJp {
    pub scraper: TcgCollectorScraper,
    pub repository: Repository,
}

impl PtcgJp {
    pub async fn update_exp(&self) -> Result<(), Error> {
        let exps = self.scraper.fetch_exps().await?;
        self.repository.save_ptcg_jp_expansions(exps).await?;
        Ok(())
    }
    pub async fn save_html(&self) -> Result<(), Error> {
        let links = self.repository.get_ptcg_jp_expansions_links().await?;
        for link in links {
            let details = self.scraper.fetch_card_detail_html(&link).await?;
            self.repository.save_tcg_collector(details).await?;
        }
        Ok(())
    }
    pub async fn update_cards(&self) -> Result<(), Error> {
        self.repository
            .get_tc_details()
            .map_err(Error::from)
            .try_for_each(|d| async move {
                let card = self.scraper.fetch_card_detail(d).await?;
                self.repository.save_ptcg_jp_cards(vec![card]).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
    pub async fn update_rarity(&self) -> Result<(), Error> {
        let links = self.repository.get_ptcg_jp_expansions_links().await?;
        for link in links {
            let rarities = self.scraper.fetch_card_rarity(&link).await?;
            self.repository.update_tc_rarity(rarities).await?;
        }
        Ok(())
    }
    pub async fn build_extra(&self) -> Result<(), Error> {
        self.repository
            .get_tc_details()
            .map_err(Error::from)
            .try_for_each(|d| async move {
                if !self.repository.pokemon().ptcg_tw_is_exists(&d).await? {
                    let card = self.scraper.fetch_card_detail(d).await?;
                    dbg!(&card);
                    // self.repository.save_ptcg_jp_cards(vec![card]).await?;
                    self.repository.pokemon().save_extra(card).await?;
                }
                Ok(())
            })
            .await?;
        Ok(())
    }
}
