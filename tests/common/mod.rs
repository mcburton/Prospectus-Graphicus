//! Shared test helpers.
#![allow(dead_code)]

use std::sync::Mutex;

use prospectus_graphicus::auctoritas::TokenStore;
use prospectus_graphicus::config::{AuthConfig, Config, GraphConfig};
use prospectus_graphicus::error::Result;

pub fn test_config(login_base: &str, graph_base: &str) -> Config {
    Config {
        auth: AuthConfig {
            client_id: "test-client-id".into(),
            tenant_id: "test-tenant".into(),
        },
        graph: GraphConfig {
            endpoint_override: Some(graph_base.to_string()),
            login_override: Some(login_base.to_string()),
        },
    }
}

/// In-memory `TokenStore` so tests don't touch the real keyring.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Option<String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_token(t: &str) -> Self {
        Self {
            inner: Mutex::new(Some(t.to_string())),
        }
    }
}

impl TokenStore for MemoryStore {
    fn save_refresh_token(&self, token: &str) -> Result<()> {
        *self.inner.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    fn load_refresh_token(&self) -> Result<Option<String>> {
        Ok(self.inner.lock().unwrap().clone())
    }

    fn clear(&self) -> Result<()> {
        *self.inner.lock().unwrap() = None;
        Ok(())
    }
}
