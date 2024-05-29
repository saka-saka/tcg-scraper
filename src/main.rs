mod application;
mod domain;
mod error;
mod export;
mod handlers;
mod repository;
mod scraper;
mod strategy;

use application::Application;
use axum::{routing::get, Router};
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use meilisearch_sdk::Client;
use serde::Deserialize;
use sqlx::PgPool;
use std::{thread::sleep, time::Duration};
use strategy::Source;
use tracing::{debug, info, Level};

use crate::handlers::{list, modal, pokemon, prepare, root, search, stylesheets, MyState};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Ptcg(PtcgCommands),
    #[command(subcommand)]
    Yugioh(YugiohCommands),
    #[command(subcommand)]
    Ws(WsCommands),
    #[command(subcommand)]
    OnePiece(OnePieceCommands),
    #[command(subcommand)]
    PtcgJp(PtcgJpCommands),
    #[command(subcommand)]
    Serve(ServeCommands),
}

#[derive(Subcommand)]
enum ServeCommands {
    Ptcg,
}

#[derive(Subcommand)]
enum PtcgJpCommands {
    Exp,
    Card,
    Tc,
    Extra,
    Rarity,
}

#[derive(Subcommand)]
enum PtcgCommands {
    Prepare,
    Run,
    ExportCsv,
    Strategy,
}

#[derive(Subcommand)]
enum WsCommands {
    Scrape,
    DownloadImages,
    ExportCsv,
}

#[derive(Subcommand)]
enum YugiohCommands {
    BuildExpLink,
    BuildPriLink,
    BuildDetail,
    ExportCsv,
}

#[derive(Subcommand)]
enum OnePieceCommands {
    Scrape,
    ScrapeProducts,
    DownloadImages,
    ExportCsv,
    ExportProductCsv,
}

#[derive(Deserialize, Debug)]
struct PtcgExpansionDbRow {
    exp: String,
    name: String,
    strategy: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL")?;
    let application = Application::new(&database_url);

    match &cli.command {
        Commands::Ptcg(commands) => match commands {
            PtcgCommands::Prepare => {
                let pokemon_trainer = application.ptcg();
                pokemon_trainer.prepare_ptcg_expansions().await?;
                // pokemon_trainer.update_ptcg_fetchable().await?;
                // pokemon_trainer.update_ptcg_printing().await?;
                // pokemon_trainer.update_rarity().await?;
                // pokemon_trainer.download_all_image().await?;
            }
            PtcgCommands::Run => {
                let pokemon_trainer = application.ptcg();
            }
            PtcgCommands::ExportCsv => {
                let pokemon_trainer = application.ptcg();
                let _all_cards = pokemon_trainer.export_pokemon_trainer().await?;
            }
            PtcgCommands::Strategy => {
                debug!("strategy ...");
                let stdin = std::io::stdin();
                let mut rdr = csv::Reader::from_reader(stdin);
                for result in rdr.deserialize() {
                    let record: PtcgExpansionDbRow = result?;
                    let ptcg = application.ptcg();
                    let sources: Vec<Source> = serde_json::from_str(&record.strategy)?;
                    ptcg.from_expansion(sources, record).await?;
                }
            }
        },
        Commands::Yugioh(YugiohCommands::BuildExpLink) => {
            application.yugioh().build_yugioh_expansion_link().await;
        }
        Commands::Yugioh(YugiohCommands::BuildPriLink) => loop {
            application
                .yugioh()
                .build_yugioh_printing_link()
                .await
                .expect("ran out of expansion link to build printing link");
            println!("done wait for 1 secs ...");
            sleep(Duration::from_secs(1));
        },
        Commands::Yugioh(YugiohCommands::BuildDetail) => loop {
            let waiting_secs = 1;
            application
                .yugioh()
                .build_yugioh_printing_detail()
                .await
                .expect("ran out of printing link");
            println!("done wait for {waiting_secs} secs ...");
            sleep(Duration::from_secs(waiting_secs));
        },
        Commands::Yugioh(YugiohCommands::ExportCsv) => {
            let wtr = std::io::stdout();
            application
                .yugioh()
                .export_yugioh_printing_detail(wtr)
                .await;
        }
        Commands::Ws(WsCommands::Scrape) => {
            let ws = application.ws();
            ws.scrape().await?;
        }
        Commands::Ws(WsCommands::ExportCsv) => {
            let wtr = std::io::stdout();
            let ws = application.ws();
            ws.export_csv(wtr).await?;
        }
        Commands::Ws(WsCommands::DownloadImages) => {
            let ws = application.ws();
            ws.download_images("asia-tcg-marketplace-dataset", "ws_images")
                .await?;
        }
        Commands::OnePiece(OnePieceCommands::Scrape) => {
            application.one_piece().scrape_one_piece().await?;
        }
        Commands::OnePiece(OnePieceCommands::DownloadImages) => {
            application.one_piece().download_images().await?;
        }
        Commands::OnePiece(OnePieceCommands::ScrapeProducts) => {
            application.one_piece().scrape_one_piece_products().await;
        }
        Commands::OnePiece(OnePieceCommands::ExportCsv) => {
            let wtr = std::io::stdout();
            application.one_piece().export_one_piece_csv(wtr).await?;
        }
        Commands::OnePiece(OnePieceCommands::ExportProductCsv) => {
            let wtr = std::io::stdout();
            application
                .one_piece()
                .export_one_piece_product_csv(wtr)
                .await?;
        }
        Commands::PtcgJp(PtcgJpCommands::Exp) => {
            let ptcg_jp = application.ptcg_jp();
            ptcg_jp.update_exp().await?;
        }
        Commands::PtcgJp(PtcgJpCommands::Card) => {
            let ptcg_jp = application.ptcg_jp();
            ptcg_jp.update_cards().await?;
        }
        Commands::PtcgJp(PtcgJpCommands::Tc) => {
            let ptcg_jp = application.ptcg_jp();
            ptcg_jp.save_html().await?;
        }
        Commands::PtcgJp(PtcgJpCommands::Extra) => {
            let ptcg_jp = application.ptcg_jp();
            ptcg_jp.build_extra().await?;
        }
        Commands::PtcgJp(PtcgJpCommands::Rarity) => {
            let ptcg_jp = application.ptcg_jp();
            ptcg_jp.update_rarity().await?;
        }
        Commands::Serve(ServeCommands::Ptcg) => {
            let meilisearch_url = std::env::var("MEILISEARCH_URL")?;
            let meilisearch_api_key = std::env::var("MEILISEARCH_API_KEY")?;
            let client = Client::new(meilisearch_url, Some(meilisearch_api_key));
            let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
            let state = MyState {
                pool,
                client,
                ptcg: application.ptcg(),
            };
            let app = Router::new()
                .route("/", get(root))
                .route("/search", get(search))
                .route("/pokemon", get(pokemon))
                .route("/modal", get(modal))
                .route("/list", get(list))
                .route("/prepare", get(prepare))
                .route("/stylesheets.css", get(stylesheets))
                .with_state(state);
            let host = "0.0.0.0:8080";
            let listener = tokio::net::TcpListener::bind(host).await?;
            info!("server listening on port {}", host);
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}
