//! Integration test: `prospectus mail list` against a `wiremock` Graph.

mod common;

use serde_json::{Value, json};
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prospectus_graphicus::auctoritas::TokenStore;
use prospectus_graphicus::commands::mail;
use prospectus_graphicus::output::Format;

#[tokio::test]
async fn mail_list_renders_json_from_graph() {
    let server = MockServer::start().await;

    // Token refresh: exchange our stored refresh token for an access token.
    Mock::given(method("POST"))
        .and(path("/test-tenant/oauth2/v2.0/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token_type": "Bearer",
            "scope": "User.Read Mail.Read",
            "access_token": "access-xyz",
            "refresh_token": "refresh-new",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&server)
        .await;

    // /me/messages: we only care that $top and $select are wired through.
    Mock::given(method("GET"))
        .and(path("/me/messages"))
        .and(query_param("$top", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "value": [
                {
                    "id": "AAMkAGNl",
                    "subject": "Welcome to Prospectus",
                    "receivedDateTime": "2026-04-19T12:34:56Z",
                    "isRead": false,
                    "from": {
                        "emailAddress": {
                            "name": "Augustus",
                            "address": "augustus@example.test"
                        }
                    }
                },
                {
                    "id": "AAMkAGNm",
                    "subject": "Reminder",
                    "receivedDateTime": "2026-04-18T09:00:00Z",
                    "isRead": true,
                    "from": {
                        "emailAddress": {
                            "address": "noreply@example.test"
                        }
                    }
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let cfg = common::test_config(&server.uri(), &server.uri());
    let http = reqwest::Client::new();
    let store = common::MemoryStore::with_token("refresh-initial");
    let mut out = Vec::new();

    mail::run_list_with_store(
        &mut out,
        mail::ListArgs {
            config: &cfg,
            http: &http,
            beta: false,
            top: 5,
            format: Format::Json,
        },
        &store,
    )
    .await
    .expect("mail list should succeed");

    let parsed: Value = serde_json::from_slice(&out).expect("output must be JSON");
    let arr = parsed.as_array().expect("output must be a JSON array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], "AAMkAGNl");
    assert_eq!(arr[0]["subject"], "Welcome to Prospectus");
    assert_eq!(arr[0]["unread"], true);
    assert_eq!(arr[0]["from"], "Augustus <augustus@example.test>");
    assert_eq!(arr[1]["from"], "noreply@example.test");
    assert_eq!(arr[1]["unread"], false);

    // The refreshed refresh token should have replaced the original.
    let saved = store.load_refresh_token().unwrap();
    assert_eq!(saved.as_deref(), Some("refresh-new"));
}

#[tokio::test]
async fn mail_list_surfaces_graph_error_with_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/test-tenant/oauth2/v2.0/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token_type": "Bearer",
            "access_token": "access-xyz",
            "refresh_token": "refresh-new",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/me/messages"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": {
                "code": "ErrorAccessDenied",
                "message": "Access is denied. Check credentials and try again.",
                "innerError": {
                    "request-id": "deadbeef-1234",
                    "date": "2026-04-19T12:34:56"
                }
            }
        })))
        .mount(&server)
        .await;

    let cfg = common::test_config(&server.uri(), &server.uri());
    let http = reqwest::Client::new();
    let store = common::MemoryStore::with_token("refresh-initial");
    let mut out = Vec::new();

    let err = mail::run_list_with_store(
        &mut out,
        mail::ListArgs {
            config: &cfg,
            http: &http,
            beta: false,
            top: 5,
            format: Format::Json,
        },
        &store,
    )
    .await
    .expect_err("403 should surface as Graph error");

    let msg = err.to_string();
    assert!(msg.contains("ErrorAccessDenied"), "got: {msg}");
    assert!(msg.contains("deadbeef-1234"), "got: {msg}");
}
