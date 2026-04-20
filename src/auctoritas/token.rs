//! Token storage and refresh.
//!
//! Refresh tokens are kept in the OS keyring. Access tokens are held only in
//! memory during a single `prospectus` invocation — we refresh on each run.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::config::Config;
use crate::error::{Error, Result};

use super::scopes;

/// Keyring service name (constant across installs).
pub const KEYRING_SERVICE: &str = "prospectus-graphicus";
/// Keyring "account" slot for the single active user.
pub const KEYRING_ACCOUNT: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: OffsetDateTime,
    pub scope: Option<String>,
    pub token_type: String,
}

impl TokenSet {
    pub fn from_oauth(resp: OAuthTokenResponse) -> Self {
        let expires_at =
            OffsetDateTime::now_utc() + time::Duration::seconds(resp.expires_in as i64);
        Self {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at,
            scope: resp.scope,
            token_type: resp.token_type,
        }
    }

    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() + time::Duration::seconds(60) >= self.expires_at
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub scope: Option<String>,
    pub token_type: String,
}

/// Abstracted keyring so tests can swap in an in-memory store.
pub trait TokenStore: Send + Sync {
    fn save_refresh_token(&self, token: &str) -> Result<()>;
    fn load_refresh_token(&self) -> Result<Option<String>>;
    fn clear(&self) -> Result<()>;
}

pub struct KeyringStore {
    entry: keyring::Entry,
}

impl KeyringStore {
    pub fn new() -> Result<Self> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?;
        Ok(Self { entry })
    }
}

impl TokenStore for KeyringStore {
    fn save_refresh_token(&self, token: &str) -> Result<()> {
        self.entry.set_password(token)?;
        Ok(())
    }

    fn load_refresh_token(&self) -> Result<Option<String>> {
        match self.entry.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn clear(&self) -> Result<()> {
        match self.entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Exchange a refresh token for a fresh access token.
pub async fn refresh(
    http: &reqwest::Client,
    cfg: &Config,
    refresh_token: &str,
) -> Result<TokenSet> {
    let url = format!(
        "{}/{}/oauth2/v2.0/token",
        cfg.login_base().trim_end_matches('/'),
        cfg.auth.tenant_id
    );
    let scope = scopes::joined();
    let params = [
        ("client_id", cfg.auth.client_id.as_str()),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("scope", scope.as_str()),
    ];
    let resp = http.post(&url).form(&params).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
        return Err(Error::OAuth2 {
            code: body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            description: body
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    let parsed: OAuthTokenResponse = resp.json().await?;
    Ok(TokenSet::from_oauth(parsed))
}

/// Acquire a valid access token: load the refresh token from the store and
/// exchange it. Returns `NotAuthenticated` if nothing is stored.
pub async fn acquire_access_token(
    http: &reqwest::Client,
    cfg: &Config,
    store: &dyn TokenStore,
) -> Result<TokenSet> {
    let rt = store.load_refresh_token()?.ok_or(Error::NotAuthenticated)?;
    let ts = refresh(http, cfg, &rt).await?;
    if let Some(new_rt) = &ts.refresh_token {
        store.save_refresh_token(new_rt)?;
    }
    Ok(ts)
}
