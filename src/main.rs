mod api;
mod boxscores;
mod leaders;
mod scores;
mod standings;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "NHL CLI Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get NHL games and scores for yesterday, today, and tomorrow
    Scores,
    /// Get current NHL standings
    Standings {
        #[arg(default_value = "wildcard")]
        /// Type of standings (division, conference, wildcard, league)
        format: String,
    },
    /// Get NHL scoring leaders
    Leaders {
        /// Type of leaders
        /// (Players: points, goals, assists, penalty-minutes, toi, plus-minus, faceoffs)
        /// (Goalies: save-percentage, goals-against-avg, shutouts, wins)
        category: String,
    },
    /// Get detailed boxscore for a specific game
    Boxscores,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match cli.command {
        Commands::Scores => {
            scores::display_scores(&client).await?;
        }
        Commands::Standings { format } => {
            standings::display_standings(&client, &format).await?;
        }
        Commands::Leaders { category } => {
            leaders::display_leaders(&client, &category).await?;
        }
        Commands::Boxscores => {
            boxscores::get_list_of_games_for_boxscores(&client).await?;
        }
    }

    Ok(())
}
