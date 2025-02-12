use colored::Colorize;
use serde_json::Value;
use crate::api::{NHL_API_URL, nhl_api_request};

pub async fn display_standings(
    client: &reqwest::Client,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/standings/now", NHL_API_URL);
    let standings = nhl_api_request(client, &url).await?;
    let separator = "-".repeat(52);
    let header = format!(
        "{:<22} {:>3} {:>3} {:>3} {:>3} {:>3} {:>6}",
        "Team", "GP", "W", "L", "OTL", "PTS", "PCT"
    );

    match format.to_lowercase().as_str() {
        "wildcard" => {
            if let Some(standings) = standings["standings"].as_array() {
                let mut teams_by_division: std::collections::HashMap<String, Vec<&Value>> =
                    std::collections::HashMap::new();
                let mut teams_by_conference: std::collections::HashMap<String, Vec<&Value>> =
                    std::collections::HashMap::new();

                // Group teams by division and conference
                for team in standings {
                    let division = team["divisionName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    let conference = team["conferenceName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    teams_by_division.entry(division).or_default().push(team);
                    teams_by_conference
                        .entry(conference)
                        .or_default()
                        .push(team);
                }

                // Sort conferences (Eastern first, then Western)
                let mut conferences: Vec<_> = teams_by_conference.keys().collect();
                conferences.sort();

                for conference in conferences {
                    println!("\n{}", separator);
                    println!(
                        "{:^52}",
                        format!("{} CONFERENCE", conference.to_uppercase())
                    );
                    println!("{}", separator);

                    // Get divisions in this conference
                    let divisions: Vec<_> = teams_by_division
                        .iter()
                        .filter(|(_, teams)| {
                            teams.first().map_or(false, |t| {
                                t["conferenceName"].as_str().unwrap_or("") == conference
                            })
                        })
                        .collect();

                    // Collect division leaders to filter out from wild card standings
                    let mut division_leaders = vec![];  

                    // Print division leaders
                    for (division_name, teams) in divisions {
                        println!("\n{} {}", division_name.bold(), "Division".bold());
                        println!("{}", header.bold().underline());

                        let mut teams = teams.clone();
                        teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));

                        // Print top 3 teams
                        for team in teams.iter().take(3) {
                            division_leaders.push(team["teamName"]["default"].as_str().unwrap_or("Unknown"));
                            print_team_stats(team);
                        }
                    }

                    // Print Wild Card standings
                    println!("\n{}", "Wild Card".bold());
                    println!("{}", header.bold().underline());

                    let mut all_teams = teams_by_conference[conference].clone();
                    all_teams.sort_by_key(|team| -(team["points"].as_i64().unwrap_or(0)));

                    // Get teams not in top 3 of their division
                    let wild_card_teams: Vec<_> = all_teams.iter().filter(|team| !division_leaders.contains(&team["teamName"]["default"].as_str().unwrap_or("Unknown"))).collect();

                    // Print wild card teams and teams outside playoff spot
                    for (i, team) in wild_card_teams.iter().enumerate() {
                        if i == 2 {
                            println!("{}", "-".repeat(52));
                        }
                        print_team_stats(team);
                    }
                }
            }
        }
        "conference" => {
            if let Some(standings) = standings["standings"].as_array() {
                let mut teams_by_conference: std::collections::HashMap<String, Vec<&Value>> =
                    std::collections::HashMap::new();

                // Group teams by conference
                for team in standings {
                    let conference = team["conferenceName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    teams_by_conference
                        .entry(conference)
                        .or_default()
                        .push(team);
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
        }
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
        }
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

    println!(
        "{:<22} {:>3} {:>3} {:>3} {:>3} {:>3} {:>6.3}",
        team_name, games_played, wins, losses, otl, points.to_string().bold(), points_pct
    );
}