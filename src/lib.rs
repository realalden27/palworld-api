//! Simple, strongly-typed async Rust client for the official
//! [Palworld Dedicated Server REST API](https://docs.palworldgame.com/api/rest-api/palwold-rest-api/).
//!
//! ## Server Configuration
//!
//! Enable the REST API in your server's `PalWorldSettings.ini`:
//!
//! ```ini
//! [/Script/Pal.PalGameUserSettings]
//! RESTAPIEnabled=True
//! RESTAPIPort=8212
//! AdminPassword="your-admin-password"
//! ```
//!
//! *(Optional)* Pass `-enable-gamedata-api` to your server launch flags to unlock
//! [`PalworldClient::game_data`] (`GET /v1/api/game-data`).
//!
//! ## Quickstart
//!
//! ```rust,no_run
//! use palworld_api::PalworldClient;
//!
//! #[tokio::main]
//! async fn main() -> palworld_api::Result<()> {
//!     let client = PalworldClient::new("http://127.0.0.1:8212", "your-admin-password")?;
//!
//!     let info = client.server_info().await?;
//!     println!("{} ({})", info.server_name, info.version);
//!
//!     let metrics = client.server_metrics().await?;
//!     println!(
//!         "Players: {}/{} | FPS: {}",
//!         metrics.current_player_num, metrics.max_player_num, metrics.server_fps
//!     );
//!
//!     client.announce("World backup starting soon!").await?;
//!     client.save_world().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Security Note
//!
//! Palworld's built-in REST API uses HTTP Basic Authentication over plain HTTP by default.
//! Restrict `RESTAPIPort` to `127.0.0.1`, a private LAN/VPN, or place it behind an
//! authenticated HTTPS reverse proxy before accessing it across untrusted networks.

mod client;
mod error;
mod secret;
mod types;

pub use client::{PalworldClient, PalworldClientBuilder};
pub use error::{Error, Result};
pub use secret::SecretPassword;
pub use types::{
    CharacterActor, PalBoxActor, Player, ServerInfo, ServerMetrics, ServerSettings, UnitType,
    WorldActor, WorldSnapshot,
};
