//! Error types for Prospectus Graphicus.
//!
//! `GraphError` preserves the `code`, `message`, and `requestId` returned by
//! Microsoft Graph so users can debug issues without spelunking through logs.

use std::fmt;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("no cached credentials found; run `prospectus auth login`")]
    NotAuthenticated,

    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("authorization pending")]
    AuthorizationPending,

    #[error("authorization declined by user")]
    AuthorizationDeclined,

    #[error("device code expired; please retry `prospectus auth login`")]
    DeviceCodeExpired,

    #[error("OAuth2 error: {code}: {description}")]
    OAuth2 { code: String, description: String },

    #[error("Graph API error: {0}")]
    Graph(#[from] GraphError),
}

/// A structured error returned by Microsoft Graph, preserving its debugging
/// metadata (`code` and `requestId`) as advertised in the Graph docs.
#[derive(Debug, Error)]
pub struct GraphError {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.status, self.code, self.message)?;
        if let Some(rid) = &self.request_id {
            write!(f, " (requestId: {rid})")?;
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
