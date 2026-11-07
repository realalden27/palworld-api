use std::env;
use std::time::Duration;

use palworld_api::PalworldClient;

#[tokio::main]
async fn main() -> palworld_api::Result<()> {
    let url = env::var("PALWORLD_URL").unwrap_or_else(|_| "http://127.0.0.1:8212".to_owned());
    let Some(password) = env::var("PALWORLD_PASSWORD").ok() else {
        println!("Set PALWORLD_PASSWORD (and optional PALWORLD_URL) to run admin actions.");
        return Ok(());
    };

    let client = PalworldClient::builder(url, password)
        .timeout(Duration::from_secs(15))
        .build()?;

    client
        .announce("Automated world save in progress...")
        .await?;
    client.save_world().await?;
    println!("World saved successfully.");

    if let Ok(kick_id) = env::var("PALWORLD_KICK_USERID") {
        client
            .kick_player(&kick_id, Some("Removed by server administrator."))
            .await?;
        println!("Kicked player {kick_id}.");
    }

    Ok(())
}
