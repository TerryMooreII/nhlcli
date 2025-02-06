use clap::{Parser, Subcommand};
use serde_json::Value;
use chrono::{Local, Duration, NaiveDate};
use colored::Colorize;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;

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
        #[arg(default_value = "points")]
        /// Type of leaders
        /// (Players: points, goals, assists, penalty-minutes, toi, plus-minus, faceoffs)
        /// (Goalies: save-percentage, goals-against-avg, shutouts, wins)
        category: String,
    },
    /// Get detailed boxscore for a specific game
    Boxscores
}

const NHL_API_URL: &'static str = "https://api-web.nhle.com/v1";

async fn nhl_api_request(client: &reqwest::Client, url: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    let json: Value = response.json().await?;
    Ok(json)
}

async fn display_scores(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let yesterday = (Local::now() - Duration::days(1)).format("%Y-%m-%d").to_string();
    let url = format!("{}/schedule/{}", NHL_API_URL, yesterday);
    let schedule = nhl_api_request(&client, &url).await?;
    let separator = "-".repeat(52);
    
    for idx in 0..schedule["gameWeek"].as_array().unwrap().len().min(3) {
        if let Some(games) = schedule["gameWeek"][idx]["games"].as_array() {
            println!("\n{}", separator);
            let date_str = schedule["gameWeek"][idx]["date"].as_str().unwrap_or("Unknown");
            
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
                let away = &game["awayTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
                let home = &game["homeTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
                let away_score = &game["awayTeam"]["score"].as_i64().unwrap_or(0);
                let home_score = &game["homeTeam"]["score"].as_i64().unwrap_or(0);

                if away_score > home_score {
                    println!("{:>18} {:>2} vs {:<2} {}", 
                        away,
                        away_score,
                        home_score.to_string().green().bold(),
                        home.green().bold());
                } else if away_score < home_score {
                    println!("{:>18} {:>2} vs {:<2} {}", 
                        away.green().bold(),
                        away_score.to_string().green().bold(),
                        home_score,
                        home);
                } else {
                    println!("{:>18} {:>2} vs {:<2} {}", 
                        away,
                        away_score,
                        home_score,
                        home);
                }
            }
        }
    }
    Ok(())
}

async fn display_standings(client: &reqwest::Client, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/standings/now", NHL_API_URL);
    let standings = nhl_api_request(client, &url).await?;
    let separator = "-".repeat(52);
    let header = format!("{:<22} {:>3} {:>3} {:>3} {:>3} {:>3} {:>6}", 
        "Team", "GP", "W", "L", "OTL", "PTS", "PCT");

    match format.to_lowercase().as_str() {
        "wildcard" => {
            if let Some(standings) = standings["standings"].as_array() {
                let mut teams_by_division: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
                let mut teams_by_conference: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
                
                // Group teams by division and conference
                for team in standings {
                    let division = team["divisionName"].as_str().unwrap_or("Unknown").to_string();
                    let conference = team["conferenceName"].as_str().unwrap_or("Unknown").to_string();
                    teams_by_division.entry(division).or_default().push(team);
                    teams_by_conference.entry(conference).or_default().push(team);
                }

                // Sort conferences (Eastern first, then Western)
                let mut conferences: Vec<_> = teams_by_conference.keys().collect();
                conferences.sort();

                for conference in conferences {
                    println!("\n{}", separator);
                    println!("{:^52}", format!("{} CONFERENCE", conference.to_uppercase()));
                    println!("{}", separator);

                    // Get divisions in this conference
                    let divisions: Vec<_> = teams_by_division.iter()
                        .filter(|(_, teams)| teams.first().map_or(false, |t| 
                            t["conferenceName"].as_str().unwrap_or("") == conference))
                        .collect();

                      // Print division leaders
                    for (division_name, teams) in divisions {
                        println!("\n{} {}", division_name.bold(), "Division".bold());
                        println!("{}", header.bold().underline());
                        
                        let mut teams = teams.clone();
                        teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));
                        
                        // Print top 3 teams
                        for team in teams.iter().take(3) {
                            print_team_stats(team);
                        }
                    }

                    // Print Wild Card standings
                    println!("\n{}", "Wild Card".bold());
                    println!("{}", header.bold().underline());
                    
                    let mut all_teams = teams_by_conference[conference].clone();
                    all_teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));
                    
                    // Get teams not in top 3 of their division
                    let wild_card_teams: Vec<_> = all_teams.iter().collect();

                    // Print wild card teams and teams outside playoff spot
                    for (i, team) in wild_card_teams.iter().skip(6).enumerate() {
                        if i == 2 {
                            println!("{}", "-".repeat(52));
                        }
                        print_team_stats(team);
                    }
                }
            }
        },
        "conference" => {
            if let Some(standings) = standings["standings"].as_array() {
                let mut teams_by_conference: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
                
                // Group teams by conference
                for team in standings {
                    let conference = team["conferenceName"].as_str().unwrap_or("Unknown").to_string();
                    teams_by_conference.entry(conference).or_default().push(team);
                }

                // Sort conferences (Eastern first, then Western)
                let mut conferences: Vec<_> = teams_by_conference.keys().collect();
                conferences.sort();

                for conference in conferences {
                    println!("\n{}", separator);
                    println!("{:^52}", conference.to_uppercase());
                    println!("{}", separator);
                    println!("{}", header.bold().underline());

                    if let Some(teams) = teams_by_conference.get(conference) {
                        let mut teams = teams.clone();
                        teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));
                        
                        for team in teams {
                            print_team_stats(team);
                        }
                    }
                }
            }
        },
        "league" => {
            if let Some(standings) = standings["standings"].as_array() {
                println!("\n{}", separator);
                println!("{:^52}", "NHL STANDINGS");
                println!("{}", separator);
                println!("{}", header.bold().underline());

                let mut teams: Vec<_> = standings.iter().collect();
                teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));

                for team in teams {
                    print_team_stats(team);
                }
            }
        },
        _ => {
            println!("Invalid format. Use 'conference', 'wildcard', or 'league'");
        }
    }
    
    Ok(())
}

fn print_team_stats(team: &Value) {
    let team_name = team["teamName"]["default"].as_str().unwrap_or("Unknown");
    let games_played = team["gamesPlayed"].as_i64().unwrap_or(0);
    let wins = team["wins"].as_i64().unwrap_or(0);
    let losses = team["losses"].as_i64().unwrap_or(0);
    let otl = team["otLosses"].as_i64().unwrap_or(0);
    let points = team["points"].as_i64().unwrap_or(0);
    let points_pct = team["pointPctg"].as_f64().unwrap_or(0.0);
    
    println!("{:<22} {:>3} {:>3} {:>3} {:>3} {:>3} {:>6.3}", 
        team_name,
        games_played,
        wins,
        losses,
        otl,
        points,
        points_pct);
}

async fn display_leaders(client: &reqwest::Client, category: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        "toi" => ("Player Time On Ice Leaders", "toi", "Time On Ice",   player_url),
        "plus-minus" => ("Player Plus Minus Leaders", "plusMinus", "Plus Minus", player_url),
        "penalty-minutes" => ("Player Penalty Minutes Leaders", "penaltyMins", "Minutes", player_url),
        "faceoffs" => ("Player Faceoff Leaders", "faceoffLeaders", "Faceoffs", player_url),
        // Goalies
        "save-percentage" => ("Goalie Save Percentage Leaders", "savePctg", "Save %", goalie_url),
        "goals-against-avg" => ("Goalie Goals Against Average Leaders", "goalsAgainstAverage", "GAA", goalie_url),
        "shutouts" => ("Goalie Shutouts Leaders", "shutouts", "Shutouts", goalie_url),
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
    println!("{:<4} {:<25} {:<20} {:>8}", 
        "Rank",
        "Player",
        "Team",
        label);
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

            println!("{:<4} {:<25} {:<20} {:>8}", 
                (i + 1),
                player_name,
                team_name,
                value_formatted.to_string().green().bold()
            );
        }
    }

    Ok(())
}

async fn display_boxscore(client: &reqwest::Client, game_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/gamecenter/{}/landing", NHL_API_URL, game_id);
    let game = nhl_api_request(client, &url).await?;
    let separator = "=".repeat(70);
    
    // Game Info
    println!("\n{}", separator);
    let away_team = game["awayTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
    let home_team = game["homeTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
    let game_date = game["gameDate"].as_str().unwrap_or("Unknown Date");
    let game_state = game["gameState"].as_str().unwrap_or("");
    let period_num = game["periodDescriptor"]["number"].as_i64().unwrap_or(0);
    let mut period = period_num.to_string();
    if period_num == 4 {
      period = "OT".to_string()
    } else if period_num == 5 {
      period = "OT/SO".to_string()
    }
    
    println!("{:^70}", format!("{} @ {}", away_team, home_team).bold());
    println!("{:^70}", game_date);
    
    let status = match game_state {
        "LIVE" => format!("Period {} - {}", period, game["clock"]["timeRemaining"].as_str().unwrap_or("")),
        "FINAL" => "Final".to_string(),
        "OFF" => "Final".to_string(),
        "FUT" => "Game Scheduled".to_string(),
        _ => game_state.to_string()
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
    println!("{:>20} {:>8} {:>8} {:>8} {:>8} {:>8}", 
        "", "1st", "2nd", "3rd", period, "Final");
    
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
                    let scorer = format!("{} {} ({})", 
                        goal["firstName"]["default"].as_str().unwrap_or(""),
                        goal["lastName"]["default"].as_str().unwrap_or(""),
                        goal["goalsToDate"].as_i64().unwrap_or(0));
                    
                    let mut assists = Vec::new();
                    if let Some(assist_list) = goal["assists"].as_array() {
                        for assist in assist_list {
                            assists.push(format!("{} {} ({})",
                                assist["firstName"]["default"].as_str().unwrap_or(""),
                                assist["lastName"]["default"].as_str().unwrap_or(""),
                                assist["assistsToDate"].as_i64().unwrap_or(0)));
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

async fn get_list_of_games_for_boxscores(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let yesterday = (Local::now() - Duration::days(2)).format("%Y-%m-%d").to_string();
    let url = format!("{}/schedule/{}", NHL_API_URL, yesterday);
    let schedule = nhl_api_request(client, &url).await?;

    // let separator = "-".repeat(52);
    let mut all_games = Vec::new();
    let mut display_items = Vec::new();
    for idx in 0..schedule["gameWeek"].as_array().unwrap().len().min(3) {
        if let Some(games) = schedule["gameWeek"][idx]["games"].as_array() {
          let date_str = schedule["gameWeek"][idx]["date"].as_str().unwrap_or("Unknown");
          let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
          .map(|d| d.format("%A").to_string())
          .unwrap_or_else(|_| date_str.to_string());
        for game in games {
          let away = game["awayTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
          let home = game["homeTeam"]["commonName"]["default"].as_str().unwrap_or("Unknown");
          let away_score = game["awayTeam"]["score"].as_i64().unwrap_or(0);
          let home_score = game["homeTeam"]["score"].as_i64().unwrap_or(0);
          let game_id = game["id"].as_i64().unwrap_or(0);
          let game_state = game["gameState"].as_str().unwrap_or("");

          let status = match game_state {
              "LIVE" => "LIVE".to_string(),
              "FINAL" => "Final".to_string(),
              "OFF" => "Final".to_string(),
              "FUT" => "Future".to_string(),
              _ => game_state.to_string()
          };

          let display_text = format!("{:<10} {:>18} {:>2} vs {:<2} {:<18} {:<10}  ",
              date,
              away,
              away_score,
              home_score,
              home,
              status,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match cli.command {
        Commands::Scores => {
            display_scores(&client).await?;
        },
        Commands::Standings { format } => {
            display_standings(&client, &format).await?;
        },
        Commands::Leaders { category } => {
            display_leaders(&client, &category).await?;
        },
        Commands::Boxscores  => {
            get_list_of_games_for_boxscores(&client).await?;
        }
    }

    Ok(())
} 