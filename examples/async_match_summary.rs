//! Prints who played a match and how it went, with the async client.
//!
//! ```sh
//! cargo run --example async_match_summary --features async -- 122539002
//! ```

use std::collections::HashMap;

use deadlock_api::{AccountId, Client, MatchId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id: u64 = std::env::args()
        .nth(1)
        .ok_or("usage: async_match_summary <match id>")?
        .parse()?;
    let api = Client::builder()
        .api_key(std::env::var("DEADLOCK_API_KEY").ok())
        .build()?;

    let info = api.match_metadata(MatchId(id)).await?.match_info;
    let ids: Vec<AccountId> = info.players.iter().map(|p| p.account_id).collect();
    let names: HashMap<AccountId, String> = api
        .steam_profiles(&ids)
        .await?
        .into_iter()
        .filter_map(|p| Some((p.account_id, p.personaname?)))
        .collect();

    println!(
        "match {}: {}:{:02}, winning team {}",
        info.match_id,
        info.duration_s / 60,
        info.duration_s % 60,
        info.winning_team.map_or("?".into(), |t| t.to_string()),
    );
    for p in &info.players {
        println!(
            "  team {} hero {:>3} {:<24} {}/{}/{}",
            p.team.map_or("?".into(), |t| t.to_string()),
            p.hero_id.map_or("?".into(), |h| h.to_string()),
            names.get(&p.account_id).map_or("?", String::as_str),
            p.kills.unwrap_or(0),
            p.deaths.unwrap_or(0),
            p.assists.unwrap_or(0),
        );
    }
    Ok(())
}
