use colored::Colorize;
use crate::api::{NHL_API_URL, nhl_api_request};

pub async fn display_leaders(
    client: &reqwest::Client,
    category: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_url = format!("{}/skater-stats-leaders/current", NHL_API_URL);
    let goalie_url = format!("{}/goalie-stats-leaders/current", NHL_API_URL);

    let separator = "-".repeat(60);
    // Title is the title of the leaderboard
    // Property is the property to sort by and the property from the api response
    // Label is the label to display in the leaderboard
    // Api_url is the url to get the leaderboard from
    let (title, property, label, api_url) = match category.to_lowercase().as_str() {
        // Players
        "points" => ("Player Points Leaders", "points", "Points", player_url),
        "goals" => ("Player Goal Leaders", "goals", "Goals", player_url),
        "assists" => ("Player Assist Leaders", "assists", "Assists", player_url),
        "toi" => (
            "Player Time On Ice Leaders",
            "toi",
            "Time On Ice",
            player_url,
        ),
        "plus-minus" => (
            "Player Plus Minus Leaders",
            "plusMinus",
            "Plus Minus",
            player_url,
        ),
        "penalty-minutes" => (
            "Player Penalty Minutes Leaders",
            "penaltyMins",
            "Minutes",
            player_url,
        ),
        "faceoffs" => (
            "Player Faceoff Leaders",
            "faceoffLeaders",
            "Faceoffs",
            player_url,
        ),
        // Goalies
        "save-percentage" => (
            "Goalie Save Percentage Leaders",
            "savePctg",
            "Save %",
            goalie_url,
        ),
        "goals-against-avg" => (
            "Goalie Goals Against Average Leaders",
            "goalsAgainstAverage",
            "GAA",
            goalie_url,
        ),
        "shutouts" => (
            "Goalie Shutouts Leaders",
            "shutouts",
            "Shutouts",
            goalie_url,
        ),
        "wins" => ("Goalie Wins Leaders", "wins", "Wins", goalie_url),
        _ => {
            println!("Invalid category. Use 'points', 'goals', or 'assists'");
            return Ok(());
        }
    };

    let leaders = nhl_api_request(client, &api_url).await?;

    println!("\n{}", separator);
    println!("{:^60}", title.bold());
    println!("{}", separator);
    println!("{:<4} {:<25} {:<20} {:>8}", "Rank", "Player", "Team", label);
    println!("{}", separator);

    if let Some(players) = leaders[property].as_array() {
        let mut players = players.to_vec();
        players.sort_by_key(|p| -(p[property].as_i64().unwrap_or(0)));

        for (i, player) in players.iter().take(20).enumerate() {
            let first_name = player["firstName"]["default"].as_str().unwrap_or("");
            let last_name = player["lastName"]["default"].as_str().unwrap_or("");
            let team_name = player["teamName"]["default"].as_str().unwrap_or("---");
            let value = player["value"].as_f64().unwrap_or(0.0);

            let player_name = format!("{} {}", first_name, last_name);
            let mut value_formatted = format!("{}", value);
            if property == "savePctg" {
                value_formatted = format!("{:.2}%", value * 100.0);
            }
            if property == "toi" {
                value_formatted = format!("{:.2}m", value);
            }

            println!(
                "{:<4} {:<25} {:<20} {:>8}",
                (i + 1),
                player_name,
                team_name,
                value_formatted.to_string().green().bold()
            );
        }
    }

    Ok(())
} 