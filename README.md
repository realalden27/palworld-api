# palworld-api

A Rust client library for interacting with the Palworld dedicated server REST API. [Palworld Dedicated Server REST API](https://docs.palworldgame.com/api/rest-api/palwold-rest-api/).

---

## Highlights

- **Simple Single-Crate Design** — One lightweight `PalworldClient` (`Clone + Send + Sync`) backed by `reqwest`'s connection pool with zero macro magic or actor boilerplate.
- **Complete Official API Coverage** — Supports all **12 `/v1/api` endpoints**, including **`GET /v1/api/game-data`** (`-enable-gamedata-api` live world actor snapshots).
- **Resilient Against Palworld Server Quirks** — Handles empty/plain-text `200 OK` bodies on `POST` commands, transient `null` fields on joining/crossplay players, string-or-boolean `"IsActive"` flags, and forward-compatible `ServerSettings` keys across game updates.
- **Zero-Panic & Credential-Safe (`#![forbid(unsafe_code)]`)** — Admin passwords are held in a `zeroize`-backed `SecretPassword` redacted as `[REDACTED]` in `Debug`/`Display` output.

---

## Installation

Requires **Rust 1.88+** (Edition 2024).

```toml
[dependencies]
palworld-api = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Feature Flags

| Feature | Default | Description |
| :--- | :---: | :--- |
| `rustls-tls` | Yes | Pure-Rust TLS via `rustls` (no OpenSSL dependency) |
| `native-tls` | No | OS-native TLS backend (`SChannel` / `SecureTransport` / `OpenSSL`) |

---

## Server Setup

Enable the REST API in your server's `PalWorldSettings.ini`:

```ini
[/Script/Pal.PalGameUserSettings]
RESTAPIEnabled=True
RESTAPIPort=8212
AdminPassword="your-admin-password"
```

*(Optional)* Add `-enable-gamedata-api` to your server startup flags to enable `GET /v1/api/game-data`.

---

## Quickstart

```rust,no_run
use std::time::Duration;
use palworld_api::PalworldClient;

#[tokio::main]
async fn main() -> palworld_api::Result<()> {
    let client = PalworldClient::builder("http://127.0.0.1:8212", "your-admin-password")
        .timeout(Duration::from_secs(10))
        .build()?;

    let info = client.server_info().await?;
    println!("{} ({})", info.server_name, info.version);

    let metrics = client.server_metrics().await?;
    println!(
        "Players: {}/{} | {} FPS",
        metrics.current_player_num, metrics.max_player_num, metrics.server_fps
    );

    for player in client.players().await? {
        println!("{} ({}) - Lv.{}", player.name, player.user_id, player.level);
    }

    client.announce("Saving world state...").await?;
    client.save_world().await?;

    Ok(())
}
```

---

## Supported Endpoints

| Method | Client Call | Official Route | Return Type |
| :--- | :--- | :--- | :--- |
| `GET` | `client.server_info()` | `/v1/api/info` | `Result<ServerInfo>` |
| `GET` | `client.players()` | `/v1/api/players` | `Result<Vec<Player>>` |
| `GET` | `client.server_settings()` | `/v1/api/settings` | `Result<ServerSettings>` |
| `GET` | `client.server_metrics()` | `/v1/api/metrics` | `Result<ServerMetrics>` |
| `GET` | `client.game_data()` | `/v1/api/game-data` | `Result<WorldSnapshot>` |
| `POST` | `client.announce(msg)` | `/v1/api/announce` | `Result<()>` |
| `POST` | `client.kick_player(uid, msg)` | `/v1/api/kick` | `Result<()>` |
| `POST` | `client.ban_player(uid, msg)` | `/v1/api/ban` | `Result<()>` |
| `POST` | `client.unban_player(uid)` | `/v1/api/unban` | `Result<()>` |
| `POST` | `client.save_world()` | `/v1/api/save` | `Result<()>` |
| `POST` | `client.shutdown_server(secs, msg)` | `/v1/api/shutdown` | `Result<()>` |
| `POST` | `client.stop_server()` | `/v1/api/stop` | `Result<()>` |

---

## Security Notice

Palworld's native REST API authenticates with HTTP Basic Auth over plain HTTP by default. Do **not** expose `RESTAPIPort` (`8212`) directly to the public internet—keep it restricted to `127.0.0.1`, a private LAN/WireGuard network, or put it behind an authenticated HTTPS reverse proxy.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
