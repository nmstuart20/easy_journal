use clap::{Parser, Subcommand};

mod commands;
mod config;
mod error;
mod journal;

use config::Config;
use error::Result;

#[derive(Parser)]
#[command(name = "journal")]
#[command(about = "Manage daily journal entries with mdbook", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new journal entry (default: today)
    New {
        /// Specific date (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Initialize journal structure
    Init,
    /// Start web server for mobile access
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::new();

    match cli.command {
        Some(Commands::New { date }) => {
            commands::new::run(date, &config)?;
        }
        Some(Commands::Init) => {
            commands::init::run(&config)?;
        }
        Some(Commands::Serve) => {
            commands::serve::run(&config).await?;
        }
        None => {
            // Default behavior: create today's entry
            commands::new::run(None, &config)?;
        }
    }

    Ok(())
}
