use std::fmt;

use zeroize::Zeroizing;

/// Zeroized wrapper for the Palworld `AdminPassword`.
///
/// Memory is overwritten with zeros when dropped, and both [`fmt::Debug`] and
/// [`fmt::Display`] emit `"[REDACTED]"` so credentials never leak into logs or
/// error traces.
#[derive(Clone)]
pub struct SecretPassword(Zeroizing<String>);

impl SecretPassword {
    /// Wraps a password string for authenticated API calls.
    #[must_use]
    pub fn new(password: impl Into<String>) -> Self {
        Self(Zeroizing::new(password.into()))
    }

    /// Exposes the raw password slice internally for HTTP Basic Auth encoding.
    #[must_use]
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretPassword {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretPassword {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for SecretPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"[REDACTED]\"")
    }
}

impl fmt::Display for SecretPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
