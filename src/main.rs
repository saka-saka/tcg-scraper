mod application;
mod bigweb_scraper;
mod domain;
mod pokemon_csv;
mod repository;

use application::Application;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use dotenvy::dotenv;
use pokemon_csv::PokemonCSV;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    UpdateCardset {
        #[arg(short, long)]
        all: bool,
    },
    UpdateCard {
        #[arg(short, long)]
        all: Option<bool>,
        #[arg(short, long)]
        set: Option<String>,
    },
    ExportCard {
        #[arg(short, long)]
        all: bool,
    },
    ResyncAll,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;
    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL")?;
    let application = Application::new(&database_url);
    tracing_subscriber::fmt::init();

    match &cli.command {
        Some(Commands::UpdateCard { all, set }) => {
            if let Some(all) = all {
                if *all {
                    application.update_entire_card_db().await?;
                }
            } else if let Some(set) = set {
                if !set.is_empty() {
                    application.update_single_set_card_db(set).await?;
                }
            }
        }
        Some(Commands::UpdateCardset { all }) => {
            if *all {
                application.update_entire_cardset_db().await?;
            }
        }
        Some(Commands::ExportCard { all }) => {
            if *all {
                let all_cards = application.export_entire_card_db().await?;
                let mut wtr = csv::Writer::from_writer(std::io::stdout());
                for card in all_cards {
                    let csv_card: PokemonCSV = card.into();
                    wtr.serialize(csv_card)?;
                }
                wtr.flush()?;
            }
        }
        Some(Commands::ResyncAll) => {
            application.unsync_entire_cardset_db().await?;
            application.update_entire_card_db().await?;
        }
        None => {}
    }
    Ok(())
}
