use clap::{Parser, Subcommand};

mod commands;
mod config;
mod error;
mod journal;

use config::Config;
use error::Result;

#[derive(Parser)]
#[command(version, about = "Manage daily journal entries with mdbook", long_about = None)]
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
    /// Authenticate with Google Tasks
    Auth {
        /// Provider (currently only "google")
        provider: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::new();

    match cli.command {
        Some(Commands::New { date }) => {
            commands::new::run(date, &config).await?;
        }
        Some(Commands::Init) => {
            commands::init::run(&config)?;
        }
        Some(Commands::Serve) => {
            commands::serve::run(&config).await?;
        }
        Some(Commands::Auth { provider }) => {
            if provider.to_lowercase() == "google" {
                commands::auth::run(&config).await?;
            } else {
                eprintln!("Unknown provider: {}. Use 'google'.", provider);
                std::process::exit(1);
            }
        }
        None => {
            // Default behavior: create today's entry
            commands::new::run(None, &config).await?;
        }
    }

    Ok(())
}
