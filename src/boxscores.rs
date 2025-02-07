use chrono::{Duration, Local, NaiveDate};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::api::{NHL_API_URL, nhl_api_request};

pub async fn display_boxscore(
    client: &reqwest::Client,
    game_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/gamecenter/{}/landing", NHL_API_URL, game_id);
    let game = nhl_api_request(client, &url).await?;
    let separator = "=".repeat(70);

    // Game Info
    println!("\n{}", separator);
    let away_team = game["awayTeam"]["commonName"]["default"]
        .as_str()
        .unwrap_or("Unknown");
    let home_team = game["homeTeam"]["commonName"]["default"]
        .as_str()
        .unwrap_or("Unknown");
    let game_date = game["gameDate"].as_str().unwrap_or("Unknown Date");
    let game_date_formatted = NaiveDate::parse_from_str(game_date, "%Y-%m-%d")
        .map(|d| d.format("%A, %B %d").to_string())
        .unwrap_or_else(|_| game_date.to_string());
    let game_state = game["gameState"].as_str().unwrap_or("");
    let period_num = game["periodDescriptor"]["number"].as_i64().unwrap_or(0);
    let mut period = period_num.to_string();
    if period_num == 4 {
        period = "OT".to_string()
    } else if period_num == 5 {
        period = "OT/SO".to_string()
    }

    println!("{:^70}", format!("{} @ {}", away_team, home_team).bold());
    println!("{:^70}", game_date_formatted);

    let status = match game_state {
        "LIVE" => {
            let mut label = "Period";
            let is_intermission = game["clock"]["inIntermission"].as_bool().unwrap_or(false);
            if is_intermission {
                label = "Intermission";
            }
            if period_num == 4 {
                label = "Overtime";
            }
            if period_num == 5 {
                label = "Shootout";
            }
            format!(
                "{} {} - {}",
                label,
                period,
                game["clock"]["timeRemaining"].as_str().unwrap_or("")
            )
        }
        "FINAL" => "Final".to_string(),
        "OFF" => "Final".to_string(),
        "PRE" => "Pre-Game".to_string(),
        "FUT" => "Game Scheduled".to_string(),
        _ => game_state.to_string(),
    };

    if period_num == 5 {
        println!("{:^70}", "Final - Shootout".bold());
    } else if period_num == 4 {
        println!("{:^70}", "Final - Overtime".bold());
    } else {
        println!("{:^70}", status.bold());
    }
    println!("{}", separator);

    // Score by Period
    println!("\n{:^70}", "SCORING SUMMARY".bold());
    println!("{}", "-".repeat(70));
    println!(
        "{:>20} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "", "1st", "2nd", "3rd", "OT", "Final"
    );

    // Team scoring
    if let Some(scoring) = game["summary"]["scoring"].as_array() {
        let mut away_scores = vec![0; scoring.len()];
        let mut home_scores = vec![0; scoring.len()];

        for (i, period) in scoring.iter().enumerate() {
            if let Some(goals) = period["goals"].as_array() {
                for goal in goals {
                    if goal["teamAbbrev"]["default"].as_str() == game["awayTeam"]["abbrev"].as_str() {
                        away_scores[i] += 1;
                    } else {
                        home_scores[i] += 1;
                    }
                }
            }
        }
        if away_scores.len() == 3 {
            away_scores.push(0);
        }
        if home_scores.len() == 3 {
            home_scores.push(0);
        }

        // Print away team scoring
        print!("{:>20}", away_team);
        let away_total: i32 = away_scores.iter().sum();
        if away_scores.len() == 5 {
            away_scores[3] = away_scores[3] + away_scores[4];
            away_scores.pop();
        }
        if home_scores.len() == 5 {
            home_scores[3] = home_scores[3] + home_scores[4];
            home_scores.pop();
        }
        for score in away_scores {
            print!(" {:>8}", score);
        }
        println!(" {:>8}", away_total);

        // Print home team scoring
        print!("{:>20}", home_team);
        let home_total: i32 = home_scores.iter().sum();
        for score in home_scores {
            print!(" {:>8}", score);
        }
        println!(" {:>8}", home_total);
    }

    // Scoring Details
    if let Some(scoring) = game["summary"]["scoring"].as_array() {
        println!("\n{:^70}", "SCORING PLAYS".bold());
        println!("{}", "-".repeat(70));

        for (period_idx, period) in scoring.iter().enumerate() {
            if period_idx < 3 {
                println!("\n{}", format!("Period {}", period_idx + 1).bold());
            } else if period_idx == 3 {
                println!("\n{}", format!("Overtime").bold());
            } else if period_idx == 4 {
                println!("\n{}", format!("Shootout").bold());
            }

            if let Some(goals) = period["goals"].as_array() {
                if goals.len() == 0 {
                    println!("No goals scored in this period");
                    continue;
                }
                for goal in goals {
                    let time = goal["timeInPeriod"].as_str().unwrap_or("00:00");
                    let team = goal["teamAbbrev"]["default"].as_str().unwrap_or("");
                    let scorer = format!(
                        "{} {} ({})",
                        goal["firstName"]["default"].as_str().unwrap_or(""),
                        goal["lastName"]["default"].as_str().unwrap_or(""),
                        goal["goalsToDate"].as_i64().unwrap_or(0)
                    );

                    let mut assists = Vec::new();
                    if let Some(assist_list) = goal["assists"].as_array() {
                        for assist in assist_list {
                            assists.push(format!(
                                "{} {} ({})",
                                assist["firstName"]["default"].as_str().unwrap_or(""),
                                assist["lastName"]["default"].as_str().unwrap_or(""),
                                assist["assistsToDate"].as_i64().unwrap_or(0)
                            ));
                        }
                    }

                    let assist_text = if assists.is_empty() {
                        "Unassisted".to_string()
                    } else {
                        format!("Assists: {}", assists.join(", "))
                    };

                    println!("{} {} - {} ({})", time, team, scorer, assist_text);
                }
            }
        }
    }

    Ok(())
}

pub async fn get_list_of_games_for_boxscores(
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let yesterday = (Local::now() - Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let url = format!("{}/schedule/{}", NHL_API_URL, yesterday);
    let schedule = nhl_api_request(client, &url).await?;

    let mut all_games = Vec::new();
    let mut display_items = Vec::new();
    for idx in 0..schedule["gameWeek"].as_array().unwrap().len().min(3) {
        if let Some(games) = schedule["gameWeek"][idx]["games"].as_array() {
            let date_str = schedule["gameWeek"][idx]["date"]
                .as_str()
                .unwrap_or("Unknown");
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|d| d.format("%A").to_string())
                .unwrap_or_else(|_| date_str.to_string());
            for game in games {
                let away = game["awayTeam"]["commonName"]["default"]
                    .as_str()
                    .unwrap_or("Unknown");
                let home = game["homeTeam"]["commonName"]["default"]
                    .as_str()
                    .unwrap_or("Unknown");
                let away_score = game["awayTeam"]["score"].as_i64().unwrap_or(0);
                let home_score = game["homeTeam"]["score"].as_i64().unwrap_or(0);
                let game_id = game["id"].as_i64().unwrap_or(0);
                let game_state = game["gameState"].as_str().unwrap_or("");

                let status = match game_state {
                    "LIVE" => "LIVE".to_string(),
                    "FINAL" => "Final".to_string(),
                    "OFF" => "Final".to_string(),
                    "FUT" => "Future".to_string(),
                    _ => game_state.to_string(),
                };

                let display_text = format!(
                    "{:<10} {:>18} {:>2} vs {:<2} {:<18} {:<10}  ",
                    date, away, away_score, home_score, home, status,
                );

                all_games.push(game_id);
                display_items.push(display_text);
            }
        }
    }

    all_games.reverse();
    display_items.reverse();

    if display_items.is_empty() {
        println!("No games found");
        return Ok(());
    }

    // Create selection menu
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a game to view details")
        .items(&display_items)
        .default(0)
        .interact()?;

    display_boxscore(&client, &all_games[selection].to_string()).await?;
    Ok(())
} 