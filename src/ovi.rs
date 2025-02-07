use crate::api::{NHL_API_URL, nhl_api_request};
use colored::Colorize;

pub async fn display_ovi(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/player/8471214/landing", NHL_API_URL);
    let player = nhl_api_request(&client, &url).await?;

    let total_goals = player["featuredStats"]["regularSeason"]["career"]["goals"].as_u64().unwrap_or(0);
    let gretzky_goals = 894;
    let remaining_goals = gretzky_goals - total_goals;
    let beat_gretzky = remaining_goals + 1;
    let ovi = format!("Ovi has {} goals and needs {} more to tie and {} to beat Gretzky's record of {}.", total_goals, remaining_goals, beat_gretzky, gretzky_goals);
    println!("\n{}\n", ovi.green());

    Ok(())

  }
