//! Integration test: `prospectus auth login` device-code flow end-to-end
//! against a `wiremock`-backed Microsoft identity platform.

mod common;

use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prospectus_graphicus::auctoritas::TokenStore;
use prospectus_graphicus::commands::auth;

#[tokio::test]
async fn device_code_login_stores_refresh_token() {
    let server = MockServer::start().await;

    // Step 1: /devicecode returns the user code + verification URI.
    Mock::given(method("POST"))
        .and(path("/test-tenant/oauth2/v2.0/devicecode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_code": "dev-code-abc",
            "user_code": "ABCD1234",
            "verification_uri": "https://example.test/device",
            "expires_in": 900,
            "interval": 1,
            "message": "Go to https://example.test/device and enter ABCD1234"
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Step 2: first poll returns authorization_pending, second returns success.
    Mock::given(method("POST"))
        .and(path("/test-tenant/oauth2/v2.0/token"))
        .and(body_string_contains(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code",
        ))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "authorization_pending",
            "error_description": "User hasn't finished authenticating."
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/test-tenant/oauth2/v2.0/token"))
        .and(body_string_contains(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token_type": "Bearer",
            "scope": "User.Read Mail.Read",
            "access_token": "access-xyz",
            "refresh_token": "refresh-xyz",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    let cfg = common::test_config(&server.uri(), "https://graph.example.test/v1.0");
    let http = reqwest::Client::new();
    let store = common::MemoryStore::new();
    let mut stderr = Vec::new();

    auth::run_with_store(
        &mut stderr,
        auth::LoginArgs {
            config: &cfg,
            http: &http,
        },
        &store,
    )
    .await
    .expect("login should succeed");

    let saved = store.load_refresh_token().unwrap();
    assert_eq!(saved.as_deref(), Some("refresh-xyz"));
    let log = String::from_utf8(stderr).unwrap();
    assert!(log.contains("ABCD1234") || log.contains("Go to"));
    assert!(log.contains("Login successful"));
}
