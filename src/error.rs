/// Crate-wide result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by [`crate::PalworldClient`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Invalid client configuration (such as an invalid URL, unsupported scheme,
    /// or credentials embedded directly in the URL).
    #[error("invalid client configuration: {0}")]
    InvalidConfig(String),

    /// Authentication rejected by the Palworld server (`401 Unauthorized` or `403 Forbidden`).
    #[error("authentication failed (HTTP {status})")]
    Unauthorized {
        /// HTTP status code returned by the server.
        status: u16,
    },

    /// Non-success HTTP status returned by a Palworld endpoint (`400`, `404`, `5xx`, etc.).
    #[error("API error (HTTP {status}) on {endpoint}: {body}")]
    Api {
        /// HTTP status code returned by the server.
        status: u16,
        /// Relative API route (e.g. `"/info"` or `"/game-data"`).
        endpoint: String,
        /// Response text returned by the server, if any.
        body: String,
    },

    /// Request or connection timed out before completing.
    #[error("request timed out on {endpoint}")]
    Timeout {
        /// Relative API route that timed out.
        endpoint: String,
    },

    /// Underlying HTTP transport or connection failure.
    #[error("HTTP transport error on {endpoint}: {source}")]
    Transport {
        /// Relative API route being called.
        endpoint: String,
        /// Underlying `reqwest` transport error.
        #[source]
        source: reqwest::Error,
    },

    /// Failed to decode a JSON response body from a query endpoint.
    #[error("failed to decode JSON response from {endpoint}: {source}")]
    Decode {
        /// Relative API route whose JSON response failed to parse.
        endpoint: String,
        /// Underlying `serde_json` deserialization error.
        #[source]
        source: serde_json::Error,
    },
}

impl Error {
    /// Returns `true` when the server rejected credentials (`401` / `403`).
    #[must_use]
    pub const fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized { .. })
    }

    /// Returns `true` when the request timed out.
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Returns the HTTP status code if the error originated from an HTTP status response.
    #[must_use]
    pub const fn status_code(&self) -> Option<u16> {
        match self {
            Self::Unauthorized { status } | Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }
}
