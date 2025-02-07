use chrono::{Duration, Local, NaiveDate};
use colored::Colorize;
use crate::api::{NHL_API_URL, nhl_api_request};

pub async fn display_scores(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let yesterday = (Local::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let url = format!("{}/schedule/{}", NHL_API_URL, yesterday);
    let schedule = nhl_api_request(&client, &url).await?;
    let separator = "-".repeat(52);

    for idx in 0..schedule["gameWeek"].as_array().unwrap().len().min(3) {
        if let Some(games) = schedule["gameWeek"][idx]["games"].as_array() {
            println!("\n{}", separator);
            let date_str = schedule["gameWeek"][idx]["date"]
                .as_str()
                .unwrap_or("Unknown");

            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|d| d.format("%A, %B %d").to_string())
                .unwrap_or_else(|_| date_str.to_string());

            println!("{:^52}", date);
            println!("{}", separator);

            if games.len() == 0 {
                println!("{:^52}", "No games scheduled for today");
                continue;
            }

            for game in games {
                let away = &game["awayTeam"]["commonName"]["default"]
                    .as_str()
                    .unwrap_or("Unknown");
                let home = &game["homeTeam"]["commonName"]["default"]
                    .as_str()
                    .unwrap_or("Unknown");
                let away_score = &game["awayTeam"]["score"].as_i64().unwrap_or(0);
                let home_score = &game["homeTeam"]["score"].as_i64().unwrap_or(0);

                if away_score > home_score {
                    println!(
                        "{:>18} {:>2} vs {:<2} {}",
                        away.green().bold(),
                        away_score.to_string().green().bold(),
                        home_score,
                        home
                    );
                } else if away_score < home_score {
                    println!(
                        "{:>18} {:>2} vs {:<2} {}",
                        away,
                        away_score,
                        home_score.to_string().green().bold(),
                        home.green().bold()
                    );
                } else {
                    println!(
                        "{:>18} {:>2} vs {:<2} {}",
                        away, away_score, home_score, home
                    );
                }
            }
        }
    }
    Ok(())
} 