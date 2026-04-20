//! OAuth2 device authorization grant against the Microsoft identity platform.
//!
//! Reference:
//! <https://learn.microsoft.com/entra/identity-platform/v2-oauth2-device-code>

use std::time::Duration;

use serde::Deserialize;
use tokio::time::sleep;

use crate::config::Config;
use crate::error::{Error, Result};

use super::scopes;
use super::token::{OAuthTokenResponse, TokenSet, TokenStore};

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: Option<String>,
}

/// Request a device code. Caller displays `user_code` + `verification_uri`
/// (or the pre-formatted `message`) to the user.
pub async fn request_device_code(
    http: &reqwest::Client,
    cfg: &Config,
) -> Result<DeviceCodeResponse> {
    let url = format!(
        "{}/{}/oauth2/v2.0/devicecode",
        cfg.login_base().trim_end_matches('/'),
        cfg.auth.tenant_id
    );
    let scope = scopes::joined();
    let params = [
        ("client_id", cfg.auth.client_id.as_str()),
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
    Ok(resp.json().await?)
}

/// Poll the token endpoint until the user completes the flow or the code
/// expires. Honors server-specified `interval` and `slow_down` backoff.
pub async fn poll_for_token(
    http: &reqwest::Client,
    cfg: &Config,
    device: &DeviceCodeResponse,
) -> Result<TokenSet> {
    let url = format!(
        "{}/{}/oauth2/v2.0/token",
        cfg.login_base().trim_end_matches('/'),
        cfg.auth.tenant_id
    );
    let mut interval = Duration::from_secs(device.interval.max(1));
    let deadline = std::time::Instant::now() + Duration::from_secs(device.expires_in.max(60));

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(Error::DeviceCodeExpired);
        }
        sleep(interval).await;

        let params = [
            ("client_id", cfg.auth.client_id.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device.device_code.as_str()),
        ];
        let resp = http.post(&url).form(&params).send().await?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);

        if status.is_success() {
            let parsed: OAuthTokenResponse = serde_json::from_value(body)?;
            return Ok(TokenSet::from_oauth(parsed));
        }

        let code = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        match code.as_str() {
            "authorization_pending" => continue,
            "slow_down" => {
                interval += Duration::from_secs(5);
                continue;
            }
            "authorization_declined" => return Err(Error::AuthorizationDeclined),
            "expired_token" | "code_expired" => return Err(Error::DeviceCodeExpired),
            _ => {
                return Err(Error::OAuth2 {
                    code,
                    description: body
                        .get("error_description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }
}

/// End-to-end interactive login: request a device code, surface it to the
/// caller via `on_code`, poll, and persist the refresh token.
pub async fn login_interactive<F>(
    http: &reqwest::Client,
    cfg: &Config,
    store: &dyn TokenStore,
    on_code: F,
) -> Result<TokenSet>
where
    F: FnOnce(&DeviceCodeResponse),
{
    let device = request_device_code(http, cfg).await?;
    on_code(&device);
    let token = poll_for_token(http, cfg, &device).await?;
    if let Some(rt) = &token.refresh_token {
        store.save_refresh_token(rt)?;
    }
    Ok(token)
}
