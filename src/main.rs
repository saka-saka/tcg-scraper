mod application;
mod domain;
mod error;
mod export;
mod repository;
mod scraper;

use application::Application;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use dotenvy::dotenv;
use std::{thread::sleep, time::Duration};
use tracing::Level;

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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;
    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL")?;
    let application = Application::new(&database_url);
    tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .finish();

    match &cli.command {
        Commands::Ptcg(commands) => match commands {
            PtcgCommands::Prepare => {
                let pokemon_trainer = application.pokemon_trainer();
                pokemon_trainer.update_ptcg_expansion().await?;
                pokemon_trainer.update_ptcg_fetchable().await?;
                pokemon_trainer.update_ptcg_printing().await?;
                pokemon_trainer.download_all_image().await?;
            }
            PtcgCommands::Run => {
                let pokemon_trainer = application.pokemon_trainer();
                pokemon_trainer.update_rarity().await?;
            }
            PtcgCommands::ExportCsv => {
                let pokemon_trainer = application.pokemon_trainer();
                let _all_cards = pokemon_trainer.export_pokemon_trainer().await?;
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
            ws.download_images().await?;
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
    }
    Ok(())
}
