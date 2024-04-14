use crate::{bigweb_scraper::BigwebScraper, repository::Repository};

pub struct Bigweb {
    pub scraper: BigwebScraper,
    pub repository: Repository,
}

impl Bigweb {
    pub async fn update_entire_cardset_db(&self) -> Result<(), crate::error::Error> {
        let pokemon_cardsets = &self.scraper.fetch_pokemon_cardset()?;
        dbg!(pokemon_cardsets);
        // let (sets, errs): (Vec<_>, Vec<_>) =
        //     pokemon_cardsets
        //         .iter()
        //         .fold((vec![], vec![]), |mut acc, elem| {
        //             match elem {
        //                 Ok(result) => acc.0.push(result),
        //                 Err(err) => acc.1.push(err),
        //             };
        //             acc
        //         });
        // for err in errs {
        //     error!(?err)
        // }
        // for cardset in sets {
        //     debug!(?cardset);
        //     self.repository.upsert_cardset(cardset).await?;
        // }
        Ok(())
    }
}
