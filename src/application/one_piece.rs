use crate::{
    one_piece_csv::{OnePieceCsv, OnePieceProductsCsv},
    one_piece_scraper::OnePieceScraper,
    repository::Repository,
};

pub struct OnePiece {
    pub scraper: OnePieceScraper,
    pub repository: Repository,
}
impl OnePiece {
    pub async fn scrape_one_piece(&self) {
        let sets = self.scraper.set().await;
        for set in sets {
            for card in self.scraper.scrape_cards(&set).await.unwrap() {
                // let c: OnePieceCsv = card.unwrap().into();
                self.repository.upsert_one_piece(card.unwrap()).await;
            }
        }
    }
    pub async fn scrape_one_piece_products(&self) {
        let products = self.scraper.products().await;
        println!("{:#?}", products);
    }
    pub async fn export_one_piece_product_csv<W: std::io::Write>(&self, w: W) {
        let products = self.scraper.products().await;
        let mut wtr = csv::Writer::from_writer(w);
        for product in products {
            let c: OnePieceProductsCsv = product.into();
            wtr.serialize(c).unwrap();
        }
        wtr.flush().unwrap();
    }
    pub async fn export_one_piece_csv<W: std::io::Write>(&self, w: W) {
        let sets = self.scraper.set().await;
        let mut wtr = csv::Writer::from_writer(w);
        for set in sets {
            for card in self.scraper.scrape_cards(&set).await.unwrap() {
                let c: OnePieceCsv = card.unwrap().into();
                // println!("{:?}", c);
                wtr.serialize(c).unwrap();
            }
        }
        wtr.flush().unwrap();
    }
}
