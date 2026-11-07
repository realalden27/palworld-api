#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use std::time::Duration;

use palworld_api::{Error, PalworldClient, SecretPassword, UnitType, WorldActor};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct RecordedRequest {
    method: String,
    path: String,
    auth_header: Option<String>,
    content_length_header: Option<String>,
    body: String,
}

/// Spawns a single-exchange in-process HTTP/1.1 mock server on `127.0.0.1:0`
/// and returns `(base_url, join_handle_yielding_recorded_request)`.
async fn spawn_mock_once(
    status_line: &'static str,
    content_type: Option<&'static str>,
    response_body: &'static str,
    delay: Duration,
) -> (String, tokio::task::JoinHandle<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = Vec::with_capacity(4096);
        let mut chunk = [0_u8; 1024];

        let header_end = loop {
            let n = socket.read(&mut chunk).await.unwrap();
            assert!(n > 0, "client closed socket before completing HTTP headers");
            buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
        };

        let header_text = String::from_utf8_lossy(&buf[..header_end]).into_owned();
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().unwrap();
        let mut req_parts = request_line.split_whitespace();
        let method = req_parts.next().unwrap().to_owned();
        let path = req_parts.next().unwrap().to_owned();

        let mut auth_header = None;
        let mut content_length_header = None;
        let mut content_length = 0_usize;
        for line in lines {
            if let Some((name, value)) = line.split_once(':') {
                let value = value.trim();
                if name.eq_ignore_ascii_case("authorization") {
                    auth_header = Some(value.to_owned());
                } else if name.eq_ignore_ascii_case("content-length") {
                    content_length_header = Some(value.to_owned());
                    content_length = value.parse::<usize>().unwrap();
                }
            }
        }

        while buf.len() < header_end + content_length {
            let n = socket.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }

        let body =
            String::from_utf8_lossy(&buf[header_end..header_end + content_length]).into_owned();

        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        let ct_header = match content_type {
            Some(ct) => format!("Content-Type: {ct}\r\n"),
            None => String::new(),
        };
        let raw_response = format!(
            "HTTP/1.1 {status_line}\r\n{ct_header}Content-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        let _ = socket.write_all(raw_response.as_bytes()).await;
        let _ = socket.shutdown().await;

        RecordedRequest {
            method,
            path,
            auth_header,
            content_length_header,
            body,
        }
    });

    (format!("http://{addr}"), handle)
}

#[test]
fn base_url_normalization_and_validation() {
    let c1 = PalworldClient::new("http://127.0.0.1:8212", "pw").unwrap();
    assert_eq!(c1.api_base_url(), "http://127.0.0.1:8212/v1/api");

    let c2 = PalworldClient::new("http://127.0.0.1:8212/", "pw").unwrap();
    assert_eq!(c2.api_base_url(), "http://127.0.0.1:8212/v1/api");

    let c3 = PalworldClient::new("http://127.0.0.1:8212/v1/api/", "pw").unwrap();
    assert_eq!(c3.api_base_url(), "http://127.0.0.1:8212/v1/api");

    let c4 = PalworldClient::new("https://pal.example.com/node1/", "pw").unwrap();
    assert_eq!(c4.api_base_url(), "https://pal.example.com/node1/v1/api");

    assert!(matches!(
        PalworldClient::new("", "pw"),
        Err(Error::InvalidConfig(_))
    ));
    assert!(matches!(
        PalworldClient::new("ws://127.0.0.1:8212", "pw"),
        Err(Error::InvalidConfig(_))
    ));
    assert!(matches!(
        PalworldClient::new("http://admin:s3cr3t@127.0.0.1:8212", "pw"),
        Err(Error::InvalidConfig(_))
    ));
    assert!(matches!(
        PalworldClient::builder("http://127.0.0.1:8212", "pw")
            .username("   ")
            .build(),
        Err(Error::InvalidConfig(_))
    ));
}

#[test]
fn secret_password_is_redacted_everywhere() {
    let secret = SecretPassword::new("super-secret-pal-pw");
    assert_eq!(format!("{secret}"), "[REDACTED]");
    assert_eq!(format!("{secret:?}"), "\"[REDACTED]\"");

    let builder = PalworldClient::builder("http://127.0.0.1:8212", "super-secret-pal-pw");
    let builder_dbg = format!("{builder:?}");
    assert!(!builder_dbg.contains("super-secret-pal-pw"));
    assert!(builder_dbg.contains("[REDACTED]"));

    let client = builder.build().unwrap();
    let client_dbg = format!("{client:?}");
    assert!(!client_dbg.contains("super-secret-pal-pw"));
    assert!(client_dbg.contains("[REDACTED]"));
}

#[tokio::test]
async fn server_info_endpoint_and_basic_auth() {
    let json = r#"{
        "version": "v1.0.4.0",
        "servername": "Palworld Sanctuary",
        "description": "Co-op world",
        "worldguid": "A7E97BAA767DB9029EF013BB71E993A0"
    }"#;
    let (url, handle) =
        spawn_mock_once("200 OK", Some("application/json"), json, Duration::ZERO).await;

    // "admin:palpass" -> base64 "YWRtaW46cGFscGFzcw=="
    let client = PalworldClient::new(format!("{url}/"), "palpass").unwrap();
    let info = client.server_info().await.unwrap();

    assert_eq!(info.version, "v1.0.4.0");
    assert_eq!(info.server_name, "Palworld Sanctuary");
    assert_eq!(info.description, "Co-op world");
    assert_eq!(info.world_guid, "A7E97BAA767DB9029EF013BB71E993A0");

    let recorded = handle.await.unwrap();
    assert_eq!(recorded.method, "GET");
    assert_eq!(recorded.path, "/v1/api/info");
    assert_eq!(
        recorded.auth_header.as_deref(),
        Some("Basic YWRtaW46cGFscGFzcw==")
    );
}

#[tokio::test]
async fn players_endpoint_handles_normal_and_transitional_null_fields() {
    let json = r#"{
        "players": [
            {
                "name": "PalUser",
                "accountName": "paluser",
                "playerId": "AFAFD830000000000000000000000000",
                "userId": "steam_00000000000000000",
                "ip": "127.0.0.1",
                "ping": 3.14,
                "location_x": 123.45,
                "location_y": -67.89,
                "level": 55,
                "building_count": 119
            },
            {
                "name": "LoadingCrossplayPlayer",
                "accountName": null,
                "playerId": null,
                "userId": null,
                "ip": "10.0.0.5",
                "ping": null,
                "location_x": null,
                "location_y": null,
                "level": null,
                "building_count": null
            }
        ]
    }"#;
    let (url, handle) =
        spawn_mock_once("200 OK", Some("application/json"), json, Duration::ZERO).await;

    let client = PalworldClient::new(url, "pw").unwrap();
    let players = client.players().await.unwrap();
    assert_eq!(players.len(), 2);

    assert_eq!(players[0].name, "PalUser");
    assert_eq!(players[0].user_id, "steam_00000000000000000");
    assert_eq!(players[0].level, 55);
    assert_eq!(players[0].building_count, 119);

    assert_eq!(players[1].name, "LoadingCrossplayPlayer");
    assert_eq!(players[1].user_id, "");
    assert_eq!(players[1].level, 0);

    let recorded = handle.await.unwrap();
    assert_eq!(recorded.method, "GET");
    assert_eq!(recorded.path, "/v1/api/players");
}

#[tokio::test]
async fn metrics_and_settings_preserve_extra_keys() {
    let metrics_json = r#"{
        "serverfps": 60,
        "serverfpsaverage": 59.8,
        "currentplayernum": 8,
        "serverframetime": 16.66,
        "maxplayernum": 32,
        "uptime": 86400,
        "basecampnum": 14,
        "days": 42
    }"#;
    let (m_url, m_handle) = spawn_mock_once(
        "200 OK",
        Some("application/json"),
        metrics_json,
        Duration::ZERO,
    )
    .await;

    let client = PalworldClient::new(m_url, "pw").unwrap();
    let metrics = client.server_metrics().await.unwrap();
    assert_eq!(metrics.server_fps, 60);
    assert!((metrics.server_fps_average - 59.8).abs() < f64::EPSILON);
    assert_eq!(metrics.current_player_num, 8);
    assert_eq!(metrics.uptime, 86400);
    assert_eq!(metrics.days, 42);
    assert_eq!(m_handle.await.unwrap().path, "/v1/api/metrics");

    let settings_json = r#"{
        "Difficulty": "Hard",
        "ExpRate": 1.5,
        "PlayerStomachDecreaceRate": 0.8,
        "PlayerAutoHPRegeneRate": 1.2,
        "PlayerAutoHpRegeneRateInSleep": 2.0,
        "ServerName": "Test Realm",
        "PublicPort": 8211,
        "RESTAPIEnabled": true,
        "RESTAPIPort": 8212,
        "FuturePatchSetting": "enabled-in-v2"
    }"#;
    let (s_url, s_handle) = spawn_mock_once(
        "200 OK",
        Some("application/json"),
        settings_json,
        Duration::ZERO,
    )
    .await;

    let s_client = PalworldClient::new(s_url, "pw").unwrap();
    let settings = s_client.server_settings().await.unwrap();
    assert_eq!(settings.difficulty, "Hard");
    assert!((settings.exp_rate - 1.5).abs() < f64::EPSILON);
    assert!((settings.player_stomach_decrease_rate - 0.8).abs() < f64::EPSILON);
    assert_eq!(settings.public_port, 8211);
    assert!(settings.rest_api_enabled);
    assert_eq!(settings.rest_api_port, 8212);
    assert_eq!(
        settings
            .extra
            .get("FuturePatchSetting")
            .and_then(|v| v.as_str()),
        Some("enabled-in-v2")
    );
    assert_eq!(s_handle.await.unwrap().path, "/v1/api/settings");
}

#[tokio::test]
async fn game_data_snapshot_parses_character_palbox_and_unknown_actors() {
    let snapshot_json = r#"{
        "Time": "2026-06-17 13:00:40",
        "FPS": 91.71,
        "AverageFPS": 58.25,
        "InGameTime": "02:40",
        "InGameDays": 1,
        "CustomMeta": "preserved-via-additionalProperties",
        "ActorData": [
            {
                "Type": "Character",
                "InstanceID": "inst-1",
                "UnitType": "Player",
                "NickName": "Alden",
                "userid": "steam_123",
                "level": 50,
                "HP": 4500,
                "MaxHP": 5000,
                "LocationX": 100.0,
                "LocationY": 200.0,
                "LocationZ": 300.0,
                "IsActive": "true"
            },
            {
                "Type": "Character",
                "InstanceID": "inst-2",
                "UnitType": "NPC",
                "NickName": "Wandering Merchant",
                "LocationX": 10.0,
                "LocationY": 20.0,
                "LocationZ": 30.0,
                "IsActive": false
            },
            {
                "Type": "PalBox",
                "GuildID": "guild-1",
                "GuildName": "Explorers",
                "Class": "BP_BuildObject_PalBox_V2_C",
                "LocationX": -50.0,
                "LocationY": 75.0,
                "LocationZ": 12.5
            },
            {}
        ]
    }"#;
    let (url, handle) = spawn_mock_once(
        "200 OK",
        Some("application/json"),
        snapshot_json,
        Duration::ZERO,
    )
    .await;

    let client = PalworldClient::new(url, "pw").unwrap();
    let snapshot = client.game_data().await.unwrap();
    assert_eq!(snapshot.time, "2026-06-17 13:00:40");
    assert_eq!(snapshot.in_game_time, "02:40");
    assert_eq!(snapshot.in_game_days, 1);
    assert_eq!(
        snapshot.extra.get("CustomMeta").and_then(|v| v.as_str()),
        Some("preserved-via-additionalProperties")
    );
    assert_eq!(snapshot.actor_data.len(), 4);

    match &snapshot.actor_data[0] {
        WorldActor::Character(ch) => {
            assert_eq!(ch.unit_type, UnitType::Player);
            assert_eq!(ch.nick_name, "Alden");
            assert_eq!(ch.hp, 4500);
            assert!(ch.is_active);
        },
        other => panic!("expected Character, got {other:?}"),
    }

    match &snapshot.actor_data[1] {
        WorldActor::Character(ch) => {
            assert_eq!(ch.unit_type, UnitType::Npc);
            assert!(!ch.is_active);
        },
        other => panic!("expected Character, got {other:?}"),
    }

    match &snapshot.actor_data[2] {
        WorldActor::PalBox(pb) => {
            assert_eq!(pb.guild_name, "Explorers");
            assert!((pb.location_z - 12.5).abs() < f64::EPSILON);
        },
        other => panic!("expected PalBox, got {other:?}"),
    }

    assert!(matches!(&snapshot.actor_data[3], WorldActor::Unknown(_)));

    // Verify lossless round-trip serialization of WorldSnapshot
    let reserialized = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(reserialized["ActorData"][0]["Type"], "Character");
    assert_eq!(reserialized["ActorData"][2]["Type"], "PalBox");

    assert_eq!(handle.await.unwrap().path, "/v1/api/game-data");
}

#[tokio::test]
async fn post_action_endpoints_verify_payloads_and_empty_or_text_responses() {
    // 1. announce (plain-text "OK" response)
    let (url, h) = spawn_mock_once("200 OK", Some("text/plain"), "OK", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .announce("Reboot in 5m")
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path, "/v1/api/announce");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&req.body).unwrap(),
        serde_json::json!({ "message": "Reboot in 5m" })
    );

    // 2. kick_player with optional message = None (omits "message" key)
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .kick_player("steam_111", None)
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/kick");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&req.body).unwrap(),
        serde_json::json!({ "userid": "steam_111" })
    );

    // 3. ban_player with reason
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .ban_player("steam_222", Some("Griefing"))
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/ban");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&req.body).unwrap(),
        serde_json::json!({ "userid": "steam_222", "message": "Griefing" })
    );

    // 4. unban_player
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .unban_player("steam_222")
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/unban");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&req.body).unwrap(),
        serde_json::json!({ "userid": "steam_222" })
    );

    // 5. save_world (empty request & empty response — must send Content-Length: 0 for UE5 HTTP server)
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .save_world()
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/save");
    assert_eq!(req.content_length_header.as_deref(), Some("0"));
    assert!(req.body.is_empty());

    // 6. shutdown_server
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .shutdown_server(60, Some("Maintenance"))
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/shutdown");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&req.body).unwrap(),
        serde_json::json!({ "waittime": 60, "message": "Maintenance" })
    );

    // 7. stop_server (must send Content-Length: 0 for UE5 HTTP server)
    let (url, h) = spawn_mock_once("200 OK", None, "", Duration::ZERO).await;
    PalworldClient::new(url, "pw")
        .unwrap()
        .stop_server()
        .await
        .unwrap();
    let req = h.await.unwrap();
    assert_eq!(req.path, "/v1/api/stop");
    assert_eq!(req.content_length_header.as_deref(), Some("0"));
    assert!(req.body.is_empty());
}

#[tokio::test]
async fn error_mapping_unauthorized_api_decode_and_timeout() {
    // 401 Unauthorized
    let (url, _h) = spawn_mock_once(
        "401 Unauthorized",
        Some("text/plain"),
        "Unauthorized",
        Duration::ZERO,
    )
    .await;
    let err = PalworldClient::new(url, "wrong")
        .unwrap()
        .server_info()
        .await
        .unwrap_err();
    assert!(err.is_unauthorized());
    assert_eq!(err.status_code(), Some(401));

    // 400 Bad Request
    let (url, _h) = spawn_mock_once(
        "400 Bad Request",
        Some("text/plain"),
        "Invalid userid",
        Duration::ZERO,
    )
    .await;
    let err = PalworldClient::new(url, "pw")
        .unwrap()
        .kick_player("", None)
        .await
        .unwrap_err();
    assert_eq!(err.status_code(), Some(400));
    assert!(matches!(
        err,
        Error::Api {
            status: 400,
            ref endpoint,
            ref body,
        } if endpoint == "/kick" && body == "Invalid userid"
    ));

    // Malformed JSON on 200 OK
    let (url, _h) = spawn_mock_once(
        "200 OK",
        Some("application/json"),
        "{not-valid-json",
        Duration::ZERO,
    )
    .await;
    let err = PalworldClient::new(url, "pw")
        .unwrap()
        .server_info()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Decode { ref endpoint, .. } if endpoint == "/info"));

    // Request Timeout
    let (url, _h) = spawn_mock_once(
        "200 OK",
        Some("application/json"),
        "{}",
        Duration::from_millis(250),
    )
    .await;
    let slow_client = PalworldClient::builder(url, "pw")
        .timeout(Duration::from_millis(40))
        .build()
        .unwrap();
    let err = slow_client.server_info().await.unwrap_err();
    assert!(err.is_timeout());
}

#[tokio::test]
async fn live_server_smoke_when_env_configured() {
    let Ok(password) = std::env::var("PALWORLD_LIVE_PASSWORD") else {
        return;
    };
    let url =
        std::env::var("PALWORLD_LIVE_URL").unwrap_or_else(|_| "http://127.0.0.1:8212".to_owned());

    let client = PalworldClient::new(&url, password).unwrap();

    // Verify 401 Unauthorized mapping against live UE5 HTTP server
    let bad_client = PalworldClient::new(&url, "definitely-wrong-password").unwrap();
    let bad_err = bad_client.server_info().await.unwrap_err();
    assert!(bad_err.is_unauthorized());
    assert_eq!(bad_err.status_code(), Some(401));

    let info = client.server_info().await.unwrap();
    assert!(!info.version.is_empty());

    let metrics = client.server_metrics().await.unwrap();
    assert!(metrics.max_player_num > 0);

    let settings = client.server_settings().await.unwrap();
    assert!(settings.rest_api_enabled);

    let _players = client.players().await.unwrap();

    // Verify /v1/api/game-data against live server when -enable-gamedata-api is enabled
    match client.game_data().await {
        Ok(snapshot) => {
            assert!(!snapshot.time.is_empty());
            assert!(snapshot.fps >= 0.0);
            for actor in &snapshot.actor_data {
                match actor {
                    WorldActor::Character(ch) => {
                        assert!(!ch.class.is_empty());
                    },
                    WorldActor::PalBox(pb) => {
                        assert!(!pb.class.is_empty());
                    },
                    WorldActor::Unknown(val) => {
                        panic!("unexpected unknown WorldActor on live server: {val}");
                    },
                }
            }
        },
        Err(Error::Api { status: 404, .. }) => {},
        Err(other) => panic!("unexpected game_data error: {other}"),
    }

    client
        .announce("palworld-api Rust client live check OK!")
        .await
        .unwrap();
    client.save_world().await.unwrap();
    // Re-query over kept-alive pooled connection after POST actions
    let _info_again = client.server_info().await.unwrap();
}
