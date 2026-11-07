use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_LENGTH, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::secret::SecretPassword;
use crate::types::{
    AnnounceRequest, Player, PlayerListResponse, PlayerModerationRequest, ServerInfo,
    ServerMetrics, ServerSettings, ShutdownRequest, UnbanRequest, WorldSnapshot,
};

const DEFAULT_USERNAME: &str = "admin";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Builder for configuring a [`PalworldClient`].
pub struct PalworldClientBuilder {
    base_url: String,
    username: String,
    password: SecretPassword,
    timeout: Duration,
    connect_timeout: Duration,
    http_client: Option<reqwest::Client>,
}

impl fmt::Debug for PalworldClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PalworldClientBuilder")
            .field("base_url", &self.base_url)
            .field("username", &self.username)
            .field("password", &self.password)
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("custom_http_client", &self.http_client.is_some())
            .finish()
    }
}

impl PalworldClientBuilder {
    /// Creates a new builder targeting `base_url` with HTTP Basic Auth `password`
    /// and default username `"admin"`.
    #[must_use]
    pub fn new(base_url: impl Into<String>, password: impl Into<SecretPassword>) -> Self {
        Self {
            base_url: base_url.into(),
            username: DEFAULT_USERNAME.to_owned(),
            password: password.into(),
            timeout: DEFAULT_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            http_client: None,
        }
    }

    /// Overrides the HTTP Basic Auth username (defaults to `"admin"`).
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    /// Sets the total per-request timeout (defaults to `10s`).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the TCP/TLS connection establishment timeout (defaults to `5s`).
    #[must_use]
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Supplies a preconfigured [`reqwest::Client`] (useful for custom TLS roots or proxies).
    ///
    /// Note: when a custom [`reqwest::Client`] is provided, its own timeout settings apply.
    #[must_use]
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Validates configuration and builds the [`PalworldClient`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if `base_url` is malformed, uses a non-HTTP(S)
    /// scheme, embeds credentials directly in the URL, or if the underlying HTTP client
    /// cannot be initialized.
    pub fn build(self) -> Result<PalworldClient> {
        let api_base_url = normalize_api_base_url(&self.base_url)?;
        if self.username.trim().is_empty() {
            return Err(Error::InvalidConfig(
                "username must not be empty".to_owned(),
            ));
        }

        let http = match self.http_client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .timeout(self.timeout)
                .connect_timeout(self.connect_timeout)
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .map_err(|err| {
                    Error::InvalidConfig(format!("failed to build HTTP client: {err}"))
                })?,
        };

        Ok(PalworldClient {
            http,
            inner: Arc::new(ClientInner {
                api_base_url,
                username: self.username,
                password: self.password,
            }),
        })
    }
}

struct ClientInner {
    api_base_url: String,
    username: String,
    password: SecretPassword,
}

/// Asynchronous client for the official Palworld Dedicated Server REST API (`/v1/api`).
///
/// `PalworldClient` reuses an internal connection-pooled [`reqwest::Client`] and is
/// cheap to [`Clone`] and share across tasks (`Clone + Send + Sync`).
#[derive(Clone)]
pub struct PalworldClient {
    http: reqwest::Client,
    inner: Arc<ClientInner>,
}

impl fmt::Debug for PalworldClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PalworldClient")
            .field("api_base_url", &self.inner.api_base_url)
            .field("username", &self.inner.username)
            .field("password", &self.inner.password)
            .finish()
    }
}

impl PalworldClient {
    /// Creates a [`PalworldClient`] with default settings (`username = "admin"`,
    /// `timeout = 10s`, `connect_timeout = 5s`).
    ///
    /// `base_url` may be passed with or without a trailing `/v1/api` path
    /// (for example, `"http://127.0.0.1:8212"`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if `base_url` is invalid.
    pub fn new(base_url: impl Into<String>, password: impl Into<SecretPassword>) -> Result<Self> {
        PalworldClientBuilder::new(base_url, password).build()
    }

    /// Returns a [`PalworldClientBuilder`] for customizing username, timeouts, or TLS options.
    #[must_use]
    pub fn builder(
        base_url: impl Into<String>,
        password: impl Into<SecretPassword>,
    ) -> PalworldClientBuilder {
        PalworldClientBuilder::new(base_url, password)
    }

    /// Returns the normalized `/v1/api` base URL used for requests.
    #[must_use]
    pub fn api_base_url(&self) -> &str {
        &self.inner.api_base_url
    }

    /// Retrieves general server metadata (`GET /v1/api/info`).
    pub async fn server_info(&self) -> Result<ServerInfo> {
        self.get_json("/info").await
    }

    /// Lists all currently connected players (`GET /v1/api/players`).
    pub async fn players(&self) -> Result<Vec<Player>> {
        let wrapper: PlayerListResponse = self.get_json("/players").await?;
        Ok(wrapper.players)
    }

    /// Retrieves active server gameplay and network settings (`GET /v1/api/settings`).
    pub async fn server_settings(&self) -> Result<ServerSettings> {
        self.get_json("/settings").await
    }

    /// Retrieves live server performance metrics (`GET /v1/api/metrics`).
    pub async fn server_metrics(&self) -> Result<ServerMetrics> {
        self.get_json("/metrics").await
    }

    /// Retrieves a real-time world actor snapshot (`GET /v1/api/game-data`).
    ///
    /// Note: The Palworld Dedicated Server must be started with the
    /// `-enable-gamedata-api` launch flag to enable this endpoint.
    pub async fn game_data(&self) -> Result<WorldSnapshot> {
        self.get_json("/game-data").await
    }

    /// Broadcasts an in-game chat announcement to all connected players (`POST /v1/api/announce`).
    pub async fn announce(&self, message: impl AsRef<str>) -> Result<()> {
        let payload = AnnounceRequest {
            message: message.as_ref(),
        };
        self.post_action("/announce", Some(&payload)).await
    }

    /// Kicks a player by `user_id` (`POST /v1/api/kick`) with an optional display `message`.
    pub async fn kick_player(&self, user_id: impl AsRef<str>, message: Option<&str>) -> Result<()> {
        let payload = PlayerModerationRequest {
            userid: user_id.as_ref(),
            message,
        };
        self.post_action("/kick", Some(&payload)).await
    }

    /// Bans a player by `user_id` (`POST /v1/api/ban`) with an optional display `message`.
    pub async fn ban_player(&self, user_id: impl AsRef<str>, message: Option<&str>) -> Result<()> {
        let payload = PlayerModerationRequest {
            userid: user_id.as_ref(),
            message,
        };
        self.post_action("/ban", Some(&payload)).await
    }

    /// Lifts a ban for `user_id` (`POST /v1/api/unban`).
    pub async fn unban_player(&self, user_id: impl AsRef<str>) -> Result<()> {
        let payload = UnbanRequest {
            userid: user_id.as_ref(),
        };
        self.post_action("/unban", Some(&payload)).await
    }

    /// Triggers an immediate world save (`POST /v1/api/save`).
    pub async fn save_world(&self) -> Result<()> {
        self.post_action::<()>("/save", None).await
    }

    /// Schedules a graceful server shutdown after `wait_time_secs` seconds
    /// with an optional warning `message` (`POST /v1/api/shutdown`).
    pub async fn shutdown_server(&self, wait_time_secs: u32, message: Option<&str>) -> Result<()> {
        let payload = ShutdownRequest {
            waittime: wait_time_secs,
            message,
        };
        self.post_action("/shutdown", Some(&payload)).await
    }

    /// Forces an immediate server stop without a countdown (`POST /v1/api/stop`).
    pub async fn stop_server(&self) -> Result<()> {
        self.post_action::<()>("/stop", None).await
    }

    fn endpoint_url(&self, endpoint: &str) -> String {
        format!("{}{endpoint}", self.inner.api_base_url)
    }

    async fn send_checked(
        &self,
        endpoint: &str,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let response = request
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .basic_auth(&self.inner.username, Some(self.inner.password.expose()))
            .send()
            .await
            .map_err(|err| map_reqwest_error(endpoint, err))?;

        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let code = status.as_u16();
        if code == 401 || code == 403 {
            return Err(Error::Unauthorized { status: code });
        }

        let body = response.text().await.unwrap_or_default().trim().to_owned();
        Err(Error::Api {
            status: code,
            endpoint: endpoint.to_owned(),
            body,
        })
    }

    async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = self.endpoint_url(endpoint);
        let response = self.send_checked(endpoint, self.http.get(url)).await?;
        let bytes = response
            .bytes()
            .await
            .map_err(|err| map_reqwest_error(endpoint, err))?;
        serde_json::from_slice(&bytes).map_err(|source| Error::Decode {
            endpoint: endpoint.to_owned(),
            source,
        })
    }

    async fn post_action<B: Serialize>(&self, endpoint: &str, body: Option<&B>) -> Result<()> {
        let url = self.endpoint_url(endpoint);
        let req = match body {
            Some(payload) => self.http.post(url).json(payload),
            // Unreal Engine's HTTP server rejects POST requests missing Content-Length with HTTP 411.
            None => self
                .http
                .post(url)
                .header(CONTENT_LENGTH, HeaderValue::from_static("0"))
                .body(Vec::<u8>::new()),
        };
        let response = self.send_checked(endpoint, req).await?;
        let _ = response.bytes().await;
        Ok(())
    }
}

fn map_reqwest_error(endpoint: &str, err: reqwest::Error) -> Error {
    if err.is_timeout() {
        Error::Timeout {
            endpoint: endpoint.to_owned(),
        }
    } else {
        Error::Transport {
            endpoint: endpoint.to_owned(),
            source: err,
        }
    }
}

fn normalize_api_base_url(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidConfig(
            "base_url must not be empty".to_owned(),
        ));
    }

    let mut url = reqwest::Url::parse(trimmed)
        .map_err(|err| Error::InvalidConfig(format!("invalid base_url: {err}")))?;

    match url.scheme() {
        "http" | "https" => {},
        other => {
            return Err(Error::InvalidConfig(format!(
                "unsupported URL scheme `{other}`; expected `http` or `https`"
            )));
        },
    }

    if !url.has_host() {
        return Err(Error::InvalidConfig(
            "base_url is missing a host".to_owned(),
        ));
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(Error::InvalidConfig(
            "do not embed credentials in base_url; pass password via PalworldClient::new or builder"
                .to_owned(),
        ));
    }

    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::InvalidConfig(
            "base_url must not contain query parameters or fragments".to_owned(),
        ));
    }

    let clean_path = url.path().trim_end_matches('/');
    let api_path = if clean_path.ends_with("/v1/api") {
        clean_path.to_owned()
    } else if clean_path.is_empty() {
        "/v1/api".to_owned()
    } else {
        format!("{clean_path}/v1/api")
    };

    url.set_path(&api_path);
    Ok(url.as_str().trim_end_matches('/').to_owned())
}
