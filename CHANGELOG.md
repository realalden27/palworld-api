# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-07

### Added
- `PalworldClient` and `PalworldClientBuilder` providing connection-pooled asynchronous access to all 12 official Palworld Dedicated Server REST API (`/v1/api`) endpoints:
  - Queries: `server_info`, `players`, `server_settings`, `server_metrics`, and `game_data` (`-enable-gamedata-api`).
  - Player moderation: `kick_player`, `ban_player`, and `unban_player`.
  - Server lifecycle: `announce`, `save_world`, `shutdown_server`, and `stop_server`.
- Strongly-typed `Serialize + Deserialize` models aligned with the official `docs.palworldgame.com` REST API schemas (`ServerInfo`, `Player`, `ServerMetrics`, `ServerSettings`, `WorldSnapshot`, `WorldActor`, `CharacterActor`, `PalBoxActor`, `UnitType`) with forward-compatible `extra` maps and null-tolerant deserializers.
- `SecretPassword` (`zeroize`-backed memory clearing on drop, `[REDACTED]` `Debug`/`Display` formatting) and embedded-URL credential rejection.
- Explicit `Content-Length: 0` handling on bodiless `POST` requests (`/save`, `/stop`) for compatibility with Unreal Engine's built-in HTTP server.
- `rustls-tls` (default) and `native-tls` Cargo feature flags.
