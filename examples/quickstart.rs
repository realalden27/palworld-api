use std::env;

use palworld_api::PalworldClient;

#[tokio::main]
async fn main() -> palworld_api::Result<()> {
    let url = env::var("PALWORLD_URL").unwrap_or_else(|_| "http://127.0.0.1:8212".to_owned());
    let Some(password) = env::var("PALWORLD_PASSWORD").ok() else {
        println!("Set PALWORLD_PASSWORD (and optional PALWORLD_URL) to query a live server.");
        println!("Example:");
        println!(
            "  PALWORLD_URL=http://127.0.0.1:8212 PALWORLD_PASSWORD=secret cargo run --example quickstart"
        );
        return Ok(());
    };

    let client = PalworldClient::new(url, password)?;

    let info = client.server_info().await?;
    println!(
        "Server: {} ({}) — GUID {}",
        info.server_name, info.version, info.world_guid
    );

    let metrics = client.server_metrics().await?;
    println!(
        "Metrics: {} FPS (avg {:.1}, {:.2} ms) | Players {}/{} | Bases {} | Day {} | Uptime {}s",
        metrics.server_fps,
        metrics.server_fps_average,
        metrics.server_frame_time,
        metrics.current_player_num,
        metrics.max_player_num,
        metrics.base_camp_num,
        metrics.days,
        metrics.uptime,
    );

    let settings = client.server_settings().await?;
    println!(
        "Settings: Difficulty={} | ExpRate={} | MaxPlayers={} | ExtraKeys={}",
        settings.difficulty,
        settings.exp_rate,
        settings.server_player_max_num,
        settings.extra.len(),
    );

    let players = client.players().await?;
    println!("Online players ({}):", players.len());
    for p in &players {
        println!(
            "  - {} (account={}, uid={}, pid={}) Lv.{} ping {:.1}ms @ ({:.1}, {:.1}) buildings={}",
            p.name,
            p.account_name,
            p.user_id,
            p.player_id,
            p.level,
            p.ping,
            p.location_x,
            p.location_y,
            p.building_count,
        );
    }

    match client.game_data().await {
        Ok(snapshot) => {
            println!(
                "GameData: time={} (day {}, clock {}) | FPS={:.1} (avg {:.1}) | Actors={}",
                snapshot.time,
                snapshot.in_game_days,
                snapshot.in_game_time,
                snapshot.fps,
                snapshot.average_fps,
                snapshot.actor_data.len(),
            );
        },
        Err(err) => {
            println!("GameData unavailable (launch with -enable-gamedata-api to enable): {err}");
        },
    }

    Ok(())
}
