//! `graphus` — the Graph. An authenticated HTTP client wrapping `reqwest`.

pub mod paginator;

use reqwest::Method;
use serde::de::DeserializeOwned;

use crate::auctoritas::TokenSet;
use crate::config::Config;
use crate::error::{Error, GraphError, Result};

pub struct GraphClient {
    http: reqwest::Client,
    base: String,
    access_token: String,
}

impl GraphClient {
    pub fn new(http: reqwest::Client, cfg: &Config, beta: bool, token: &TokenSet) -> Self {
        Self {
            http,
            base: cfg.graph_base(beta).trim_end_matches('/').to_string(),
            access_token: token.access_token.clone(),
        }
    }

    /// GET `<base>/<path>?<query>` and deserialize the JSON response.
    pub async fn get_json<T, Q>(&self, path: &str, query: &Q) -> Result<T>
    where
        T: DeserializeOwned,
        Q: serde::Serialize + ?Sized,
    {
        let url = format!("{}/{}", self.base, path.trim_start_matches('/'));
        let resp = self
            .http
            .request(Method::GET, &url)
            .bearer_auth(&self.access_token)
            .query(query)
            .send()
            .await?;
        handle_response(resp).await
    }

    /// GET an absolute URL (used for `@odata.nextLink` pagination).
    pub async fn get_url<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let resp = self
            .http
            .request(Method::GET, url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;
        handle_response(resp).await
    }
}

async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp.json().await?);
    }

    let request_id = resp
        .headers()
        .get("request-id")
        .or_else(|| resp.headers().get("client-request-id"))
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    let code = body
        .pointer("/error/code")
        .and_then(|v| v.as_str())
        .unwrap_or("UnknownError")
        .to_string();
    let message = body
        .pointer("/error/message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let request_id = body
        .pointer("/error/innerError/request-id")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
        .or(request_id);

    Err(Error::Graph(GraphError {
        status: status.as_u16(),
        code,
        message,
        request_id,
    }))
}
