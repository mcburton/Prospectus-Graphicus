//! Config loading: `~/.config/prospectus/config.toml`.
//!
//! Only non-secret values live here. Refresh tokens are stored in the OS
//! keyring by [`crate::auctoritas::token`].

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Default Microsoft Graph v1.0 endpoint.
pub const GRAPH_V1: &str = "https://graph.microsoft.com/v1.0";
/// Graph beta endpoint, used when `--beta` is passed.
pub const GRAPH_BETA: &str = "https://graph.microsoft.com/beta";
/// Microsoft identity platform base.
pub const LOGIN_BASE: &str = "https://login.microsoftonline.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    #[serde(default)]
    pub graph: GraphConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Azure app registration (Application/client ID).
    pub client_id: String,
    /// AAD tenant ID or domain (e.g. `pitt.edu`).
    pub tenant_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Override the Graph base URL (primarily for tests).
    #[serde(default)]
    pub endpoint_override: Option<String>,
    /// Override the Microsoft identity base URL (primarily for tests).
    #[serde(default)]
    pub login_override: Option<String>,
}

impl Config {
    /// Load from the default XDG config path, or a placeholder template if
    /// none exists yet.
    pub fn load() -> Result<Self> {
        let path = default_config_path()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(Error::Config(format!(
                "config file not found at {}. Create it with your Azure app registration \
                 details — see docs/azure-app-registration.md",
                path.display()
            )));
        }
        let raw = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(cfg)
    }

    pub fn graph_base(&self, beta: bool) -> &str {
        if let Some(o) = &self.graph.endpoint_override {
            return o;
        }
        if beta { GRAPH_BETA } else { GRAPH_V1 }
    }

    pub fn login_base(&self) -> &str {
        self.graph.login_override.as_deref().unwrap_or(LOGIN_BASE)
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("org", "prospectus", "prospectus")
        .ok_or_else(|| Error::Config("cannot determine config directory".into()))?;
    Ok(dirs.config_dir().join("config.toml"))
}

/// Template written when `prospectus` is run without a config file.
///
/// Defaults to the **Microsoft Graph PowerShell** first-party public client
/// ID (`14d82eec-204b-4c2f-b7e8-296a70dab67e`). It is multi-tenant, supports
/// the device code flow, and covers every scope Prospectus Graphicus
/// requests. This lets the CLI work without registering a private Azure app.
/// See `docs/azure-app-registration.md` for tradeoffs and how to swap in
/// your own client ID for production use.
pub const CONFIG_TEMPLATE: &str = r#"# Prospectus Graphicus configuration.
# See docs/azure-app-registration.md for details.

[auth]
# Microsoft Graph PowerShell first-party public client ID.
# Replace with your own Azure app registration when you have one.
client_id = "14d82eec-204b-4c2f-b7e8-296a70dab67e"
tenant_id = "pitt.edu"
"#;
