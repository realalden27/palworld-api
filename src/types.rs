use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

fn null_as_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BoolOrString {
    Bool(bool),
    Str(String),
}

fn flexible_bool<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<BoolOrString>::deserialize(deserializer)? {
        Some(BoolOrString::Bool(b)) => Ok(b),
        Some(BoolOrString::Str(s)) => Ok(s.eq_ignore_ascii_case("true") || s == "1"),
        None => Ok(false),
    }
}

/// General server metadata returned by `GET /v1/api/info`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Dedicated server version string (e.g. `"v0.1.5.0"`).
    #[serde(default, deserialize_with = "null_as_default")]
    pub version: String,
    /// Configured server name.
    #[serde(rename = "servername", default, deserialize_with = "null_as_default")]
    pub server_name: String,
    /// Configured server description.
    #[serde(default, deserialize_with = "null_as_default")]
    pub description: String,
    /// Unique world GUID.
    #[serde(rename = "worldguid", default, deserialize_with = "null_as_default")]
    pub world_guid: String,
}

/// Wire wrapper returned by `GET /v1/api/players`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub(crate) struct PlayerListResponse {
    #[serde(default, deserialize_with = "null_as_default")]
    pub(crate) players: Vec<Player>,
}

/// Connected player entry returned by `GET /v1/api/players`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Player {
    /// In-game character name.
    #[serde(default, deserialize_with = "null_as_default")]
    pub name: String,
    /// Platform account name.
    #[serde(rename = "accountName", default, deserialize_with = "null_as_default")]
    pub account_name: String,
    /// Character / player UID (`playerId`).
    #[serde(rename = "playerId", default, deserialize_with = "null_as_default")]
    pub player_id: String,
    /// Platform user ID (`userId`, e.g. `"steam_00000000000000000"`), used for kick/ban/unban.
    #[serde(rename = "userId", default, deserialize_with = "null_as_default")]
    pub user_id: String,
    /// Remote IP address of the connected client.
    ///
    /// The Palworld server uses `"iP"` on the wire while the documentation lists `"ip"`.
    #[serde(alias = "iP", default, deserialize_with = "null_as_default")]
    pub ip: String,
    /// Round-trip latency in milliseconds.
    #[serde(default, deserialize_with = "null_as_default")]
    pub ping: f64,
    /// World X coordinate.
    #[serde(default, deserialize_with = "null_as_default")]
    pub location_x: f64,
    /// World Y coordinate.
    #[serde(default, deserialize_with = "null_as_default")]
    pub location_y: f64,
    /// Current character level.
    #[serde(default, deserialize_with = "null_as_default")]
    pub level: i32,
    /// Number of structures owned by the player.
    #[serde(default, deserialize_with = "null_as_default")]
    pub building_count: u32,
}

/// Live server performance metrics returned by `GET /v1/api/metrics`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ServerMetrics {
    /// Instantaneous server tick FPS.
    #[serde(rename = "serverfps", default, deserialize_with = "null_as_default")]
    pub server_fps: u32,
    /// Average server tick FPS (present on Palworld v1.0.5+).
    #[serde(
        rename = "serverfpsaverage",
        default,
        deserialize_with = "null_as_default"
    )]
    pub server_fps_average: f64,
    /// Currently connected player count.
    #[serde(
        rename = "currentplayernum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub current_player_num: u32,
    /// Server frame time in milliseconds.
    #[serde(
        rename = "serverframetime",
        default,
        deserialize_with = "null_as_default"
    )]
    pub server_frame_time: f64,
    /// Configured maximum player capacity.
    #[serde(rename = "maxplayernum", default, deserialize_with = "null_as_default")]
    pub max_player_num: u32,
    /// Server process uptime in seconds.
    #[serde(default, deserialize_with = "null_as_default")]
    pub uptime: u64,
    /// Active base camp count across the world.
    #[serde(rename = "basecampnum", default, deserialize_with = "null_as_default")]
    pub base_camp_num: u32,
    /// Elapsed in-game world days.
    #[serde(default, deserialize_with = "null_as_default")]
    pub days: u32,
}

/// Active server settings returned by `GET /v1/api/settings`.
///
/// Mirrors the official schema at `https://docs.palworldgame.com/api/rest-api/settings`,
/// while preserving any additional server keys inside [`ServerSettings::extra`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ServerSettings {
    #[serde(rename = "Difficulty", default, deserialize_with = "null_as_default")]
    pub difficulty: String,
    #[serde(
        rename = "DayTimeSpeedRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub day_time_speed_rate: f64,
    #[serde(
        rename = "NightTimeSpeedRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub night_time_speed_rate: f64,
    #[serde(rename = "ExpRate", default, deserialize_with = "null_as_default")]
    pub exp_rate: f64,
    #[serde(
        rename = "PalCaptureRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_capture_rate: f64,
    #[serde(
        rename = "PalSpawnNumRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_spawn_num_rate: f64,
    #[serde(
        rename = "PalDamageRateAttack",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_damage_rate_attack: f64,
    #[serde(
        rename = "PalDamageRateDefense",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_damage_rate_defense: f64,
    #[serde(
        rename = "PlayerDamageRateAttack",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_damage_rate_attack: f64,
    #[serde(
        rename = "PlayerDamageRateDefense",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_damage_rate_defense: f64,
    #[serde(
        rename = "PlayerStomachDecreaceRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_stomach_decrease_rate: f64,
    #[serde(
        rename = "PlayerStaminaDecreaceRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_stamina_decrease_rate: f64,
    #[serde(
        rename = "PlayerAutoHPRegeneRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_auto_hp_regen_rate: f64,
    #[serde(
        rename = "PlayerAutoHpRegeneRateInSleep",
        default,
        deserialize_with = "null_as_default"
    )]
    pub player_auto_hp_regen_rate_in_sleep: f64,
    #[serde(
        rename = "PalStomachDecreaceRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_stomach_decrease_rate: f64,
    #[serde(
        rename = "PalStaminaDecreaceRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_stamina_decrease_rate: f64,
    #[serde(
        rename = "PalAutoHPRegeneRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_auto_hp_regen_rate: f64,
    #[serde(
        rename = "PalAutoHpRegeneRateInSleep",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_auto_hp_regen_rate_in_sleep: f64,
    #[serde(
        rename = "BuildObjectDamageRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub build_object_damage_rate: f64,
    #[serde(
        rename = "BuildObjectDeteriorationDamageRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub build_object_deterioration_damage_rate: f64,
    #[serde(
        rename = "CollectionDropRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub collection_drop_rate: f64,
    #[serde(
        rename = "CollectionObjectHpRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub collection_object_hp_rate: f64,
    #[serde(
        rename = "CollectionObjectRespawnSpeedRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub collection_object_respawn_speed_rate: f64,
    #[serde(
        rename = "EnemyDropItemRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enemy_drop_item_rate: f64,
    #[serde(rename = "DeathPenalty", default, deserialize_with = "null_as_default")]
    pub death_penalty: String,
    #[serde(
        rename = "bEnablePlayerToPlayerDamage",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_player_to_player_damage: bool,
    #[serde(
        rename = "bEnableFriendlyFire",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_friendly_fire: bool,
    #[serde(
        rename = "bEnableInvaderEnemy",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_invader_enemy: bool,
    #[serde(rename = "bActiveUNKO", default, deserialize_with = "null_as_default")]
    pub active_unko: bool,
    #[serde(
        rename = "bEnableAimAssistPad",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_aim_assist_pad: bool,
    #[serde(
        rename = "bEnableAimAssistKeyboard",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_aim_assist_keyboard: bool,
    #[serde(
        rename = "DropItemMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub drop_item_max_num: u32,
    #[serde(
        rename = "DropItemMaxNum_UNKO",
        default,
        deserialize_with = "null_as_default"
    )]
    pub drop_item_max_num_unko: u32,
    #[serde(
        rename = "BaseCampMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub base_camp_max_num: u32,
    #[serde(
        rename = "BaseCampWorkerMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub base_camp_worker_max_num: u32,
    #[serde(
        rename = "DropItemAliveMaxHours",
        default,
        deserialize_with = "null_as_default"
    )]
    pub drop_item_alive_max_hours: f64,
    #[serde(
        rename = "bAutoResetGuildNoOnlinePlayers",
        default,
        deserialize_with = "null_as_default"
    )]
    pub auto_reset_guild_no_online_players: bool,
    #[serde(
        rename = "AutoResetGuildTimeNoOnlinePlayers",
        default,
        deserialize_with = "null_as_default"
    )]
    pub auto_reset_guild_time_no_online_players: f64,
    #[serde(
        rename = "GuildPlayerMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub guild_player_max_num: u32,
    #[serde(
        rename = "PalEggDefaultHatchingTime",
        default,
        deserialize_with = "null_as_default"
    )]
    pub pal_egg_default_hatching_time: f64,
    #[serde(
        rename = "WorkSpeedRate",
        default,
        deserialize_with = "null_as_default"
    )]
    pub work_speed_rate: f64,
    #[serde(rename = "bIsMultiplay", default, deserialize_with = "null_as_default")]
    pub is_multiplay: bool,
    #[serde(rename = "bIsPvP", default, deserialize_with = "null_as_default")]
    pub is_pvp: bool,
    #[serde(
        rename = "bCanPickupOtherGuildDeathPenaltyDrop",
        default,
        deserialize_with = "null_as_default"
    )]
    pub can_pickup_other_guild_death_penalty_drop: bool,
    #[serde(
        rename = "bEnableNonLoginPenalty",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_non_login_penalty: bool,
    #[serde(
        rename = "bEnableFastTravel",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_fast_travel: bool,
    #[serde(
        rename = "bIsStartLocationSelectByMap",
        default,
        deserialize_with = "null_as_default"
    )]
    pub is_start_location_select_by_map: bool,
    #[serde(
        rename = "bExistPlayerAfterLogout",
        default,
        deserialize_with = "null_as_default"
    )]
    pub exist_player_after_logout: bool,
    #[serde(
        rename = "bEnableDefenseOtherGuildPlayer",
        default,
        deserialize_with = "null_as_default"
    )]
    pub enable_defense_other_guild_player: bool,
    #[serde(
        rename = "CoopPlayerMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub coop_player_max_num: u32,
    #[serde(
        rename = "ServerPlayerMaxNum",
        default,
        deserialize_with = "null_as_default"
    )]
    pub server_player_max_num: u32,
    #[serde(rename = "ServerName", default, deserialize_with = "null_as_default")]
    pub server_name: String,
    #[serde(
        rename = "ServerDescription",
        default,
        deserialize_with = "null_as_default"
    )]
    pub server_description: String,
    #[serde(rename = "PublicPort", default, deserialize_with = "null_as_default")]
    pub public_port: u32,
    #[serde(rename = "PublicIP", default, deserialize_with = "null_as_default")]
    pub public_ip: String,
    #[serde(rename = "RCONEnabled", default, deserialize_with = "null_as_default")]
    pub rcon_enabled: bool,
    #[serde(rename = "RCONPort", default, deserialize_with = "null_as_default")]
    pub rcon_port: u32,
    #[serde(rename = "Region", default, deserialize_with = "null_as_default")]
    pub region: String,
    #[serde(rename = "bUseAuth", default, deserialize_with = "null_as_default")]
    pub use_auth: bool,
    #[serde(rename = "BanListURL", default, deserialize_with = "null_as_default")]
    pub ban_list_url: String,
    #[serde(
        rename = "RESTAPIEnabled",
        default,
        deserialize_with = "null_as_default"
    )]
    pub rest_api_enabled: bool,
    #[serde(rename = "RESTAPIPort", default, deserialize_with = "null_as_default")]
    pub rest_api_port: u32,
    #[serde(
        rename = "bShowPlayerList",
        default,
        deserialize_with = "null_as_default"
    )]
    pub show_player_list: bool,
    #[serde(
        rename = "AllowConnectPlatform",
        default,
        deserialize_with = "null_as_default"
    )]
    pub allow_connect_platform: String,
    #[serde(
        rename = "bIsUseBackupSaveData",
        default,
        deserialize_with = "null_as_default"
    )]
    pub use_backup_save_data: bool,
    #[serde(
        rename = "LogFormatType",
        default,
        deserialize_with = "null_as_default"
    )]
    pub log_format_type: String,
    /// Additional settings returned by the server beyond the documented baseline.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// World actor snapshot returned by `GET /v1/api/game-data`.
///
/// Mirrors the official schema at `https://docs.palworldgame.com/api/rest-api/game-data`
/// (requires launching the Palworld Dedicated Server with `-enable-gamedata-api`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Snapshot timestamp (`"YYYY-MM-DD HH:MM:SS"`, server local time).
    #[serde(rename = "Time", default, deserialize_with = "null_as_default")]
    pub time: String,
    /// Instantaneous server FPS.
    #[serde(rename = "FPS", default, deserialize_with = "null_as_default")]
    pub fps: f64,
    /// Average server FPS.
    #[serde(rename = "AverageFPS", default, deserialize_with = "null_as_default")]
    pub average_fps: f64,
    /// Current in-game clock time (`"HH:MM"`, emitted by Palworld v1.0.5+).
    #[serde(rename = "InGameTime", default, deserialize_with = "null_as_default")]
    pub in_game_time: String,
    /// Elapsed in-game days count (emitted by Palworld v1.0.5+).
    #[serde(rename = "InGameDays", default, deserialize_with = "null_as_default")]
    pub in_game_days: u32,
    /// Array of world actors (`Character` / `PalBox`).
    #[serde(rename = "ActorData", default, deserialize_with = "null_as_default")]
    pub actor_data: Vec<WorldActor>,
    /// Additional root metadata properties (`property name*: any` in official schema).
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Actor entry inside [`WorldSnapshot::actor_data`].
#[derive(Debug, Clone, PartialEq)]
pub enum WorldActor {
    /// Character actor (`Player`, `OtomoPal`, `BaseCampPal`, `WildPal`, or `NPC`).
    Character(Box<CharacterActor>),
    /// Guild base camp `PalBox` actor.
    PalBox(PalBoxActor),
    /// Unrecognized or untyped actor object preserved for forward compatibility.
    Unknown(serde_json::Value),
}

impl Serialize for WorldActor {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(tag = "Type")]
        enum TaggedRef<'a> {
            Character(&'a CharacterActor),
            PalBox(&'a PalBoxActor),
        }

        match self {
            Self::Character(actor) => TaggedRef::Character(actor).serialize(serializer),
            Self::PalBox(actor) => TaggedRef::PalBox(actor).serialize(serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for WorldActor {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        let actor_type = value
            .as_object()
            .and_then(|obj| obj.get("Type"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);

        match actor_type.as_deref() {
            Some("Character") => {
                if let Some(obj) = value.as_object_mut() {
                    obj.remove("Type");
                }
                serde_json::from_value::<CharacterActor>(value)
                    .map(|actor| Self::Character(Box::new(actor)))
                    .map_err(serde::de::Error::custom)
            },
            Some("PalBox") => {
                if let Some(obj) = value.as_object_mut() {
                    obj.remove("Type");
                }
                serde_json::from_value::<PalBoxActor>(value)
                    .map(Self::PalBox)
                    .map_err(serde::de::Error::custom)
            },
            _ => Ok(Self::Unknown(value)),
        }
    }
}

/// Classification of a [`CharacterActor`] in `GET /v1/api/game-data`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    /// Human player character.
    Player,
    /// Party / companion Pal (`OtomoPal`).
    OtomoPal,
    /// Worker Pal assigned to a base camp (`BaseCampPal`).
    BaseCampPal,
    /// Wild spawned Pal (`WildPal`).
    WildPal,
    /// Non-player human character (`NPC`).
    #[serde(rename = "NPC")]
    Npc,
    /// Forward-compatible fallback for unrecognized unit classifications.
    #[serde(untagged)]
    Other(String),
}

impl Default for UnitType {
    fn default() -> Self {
        Self::Other(String::new())
    }
}

/// Character entity (`Type = "Character"`) in [`WorldSnapshot`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CharacterActor {
    #[serde(rename = "InstanceID", default, deserialize_with = "null_as_default")]
    pub instance_id: String,
    #[serde(rename = "UnitType", default, deserialize_with = "null_as_default")]
    pub unit_type: UnitType,
    #[serde(rename = "NickName", default, deserialize_with = "null_as_default")]
    pub nick_name: String,
    #[serde(
        rename = "TrainerInstanceID",
        default,
        deserialize_with = "null_as_default"
    )]
    pub trainer_instance_id: String,
    #[serde(
        rename = "TrainerNickName",
        default,
        deserialize_with = "null_as_default"
    )]
    pub trainer_nick_name: String,
    #[serde(rename = "TrainerClass", default, deserialize_with = "null_as_default")]
    pub trainer_class: String,
    #[serde(rename = "userid", default, deserialize_with = "null_as_default")]
    pub user_id: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub ip: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub level: i32,
    #[serde(rename = "HP", default, deserialize_with = "null_as_default")]
    pub hp: i64,
    #[serde(rename = "MaxHP", default, deserialize_with = "null_as_default")]
    pub max_hp: i64,
    #[serde(rename = "GuildID", default, deserialize_with = "null_as_default")]
    pub guild_id: String,
    #[serde(rename = "GuildName", default, deserialize_with = "null_as_default")]
    pub guild_name: String,
    #[serde(rename = "Class", default, deserialize_with = "null_as_default")]
    pub class: String,
    #[serde(rename = "Action", default, deserialize_with = "null_as_default")]
    pub action: String,
    #[serde(rename = "AI_Action", default, deserialize_with = "null_as_default")]
    pub ai_action: String,
    #[serde(rename = "LocationX", default, deserialize_with = "null_as_default")]
    pub location_x: f64,
    #[serde(rename = "LocationY", default, deserialize_with = "null_as_default")]
    pub location_y: f64,
    #[serde(rename = "LocationZ", default, deserialize_with = "null_as_default")]
    pub location_z: f64,
    #[serde(rename = "RotationX", default, deserialize_with = "null_as_default")]
    pub rotation_x: f64,
    #[serde(rename = "RotationY", default, deserialize_with = "null_as_default")]
    pub rotation_y: f64,
    #[serde(rename = "RotationZ", default, deserialize_with = "null_as_default")]
    pub rotation_z: f64,
    #[serde(rename = "Stage", default, deserialize_with = "null_as_default")]
    pub stage: String,
    #[serde(rename = "IsActive", default, deserialize_with = "flexible_bool")]
    pub is_active: bool,
}

/// Base camp `PalBox` entity (`Type = "PalBox"`) in [`WorldSnapshot`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PalBoxActor {
    /// Base camp display name (emitted by Palworld v1.0.5+).
    #[serde(rename = "Name", default, deserialize_with = "null_as_default")]
    pub name: String,
    #[serde(rename = "GuildID", default, deserialize_with = "null_as_default")]
    pub guild_id: String,
    #[serde(rename = "GuildName", default, deserialize_with = "null_as_default")]
    pub guild_name: String,
    #[serde(rename = "Class", default, deserialize_with = "null_as_default")]
    pub class: String,
    #[serde(rename = "LocationX", default, deserialize_with = "null_as_default")]
    pub location_x: f64,
    #[serde(rename = "LocationY", default, deserialize_with = "null_as_default")]
    pub location_y: f64,
    #[serde(rename = "LocationZ", default, deserialize_with = "null_as_default")]
    pub location_z: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnnounceRequest<'a> {
    pub(crate) message: &'a str,
}

#[derive(Debug, Serialize)]
pub(crate) struct PlayerModerationRequest<'a> {
    pub(crate) userid: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub(crate) struct UnbanRequest<'a> {
    pub(crate) userid: &'a str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ShutdownRequest<'a> {
    pub(crate) waittime: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<&'a str>,
}
