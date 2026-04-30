//! `prospectus mail list`: list messages from `/me/messages`.

use std::io::Write;

use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::auctoritas::{KeyringStore, TokenStore, token};
use crate::config::Config;
use crate::error::Result;
use crate::graphus::GraphClient;
use crate::output::{self, Format};

/// `epistula` — "letter". A single mail message as surfaced by the CLI.
///
/// This is a trimmed view of the Graph `message` resource: the full object
/// has dozens of fields; we expose the subset that is useful for `list`
/// output and round-trip cleanly through JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Epistula {
    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "From")]
    #[serde(default, deserialize_with = "deserialize_from")]
    pub from: String,

    #[tabled(rename = "Subject")]
    #[serde(default)]
    pub subject: String,

    #[tabled(rename = "Received")]
    #[serde(default, rename(deserialize = "receivedDateTime"))]
    pub received: String,

    #[tabled(rename = "Unread", display_with = "display_bool")]
    #[serde(
        default,
        rename(deserialize = "isRead"),
        deserialize_with = "invert_bool"
    )]
    pub unread: bool,
}

fn display_bool(b: &bool) -> String {
    if *b { "yes".into() } else { "no".into() }
}

fn invert_bool<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<bool, D::Error> {
    let is_read = bool::deserialize(d)?;
    Ok(!is_read)
}

fn deserialize_from<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<String, D::Error> {
    let v: Option<serde_json::Value> = Option::deserialize(d)?;
    Ok(v.and_then(|obj| {
        let addr = obj
            .pointer("/emailAddress/address")
            .and_then(|s| s.as_str());
        let name = obj.pointer("/emailAddress/name").and_then(|s| s.as_str());
        match (name, addr) {
            (Some(n), Some(a)) if !n.is_empty() => Some(format!("{n} <{a}>")),
            (_, Some(a)) => Some(a.to_string()),
            _ => None,
        }
    })
    .unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct MessagesPage {
    value: Vec<Epistula>,
}

pub struct ListArgs<'a> {
    pub config: &'a Config,
    pub http: &'a reqwest::Client,
    pub beta: bool,
    pub top: u32,
    pub format: Format,
}

pub async fn run_list<W: Write>(w: &mut W, args: ListArgs<'_>) -> Result<()> {
    let store = KeyringStore::new()?;
    run_list_with_store(w, args, &store).await
}

pub async fn run_list_with_store<W: Write>(
    w: &mut W,
    args: ListArgs<'_>,
    store: &dyn TokenStore,
) -> Result<()> {
    let token = token::acquire_access_token(args.http, args.config, store).await?;
    let client = GraphClient::new(args.http.clone(), args.config, args.beta, &token);

    let query = [
        ("$top", args.top.to_string()),
        (
            "$select",
            "id,subject,from,receivedDateTime,isRead".to_string(),
        ),
        ("$orderby", "receivedDateTime desc".to_string()),
    ];
    let page: MessagesPage = client.get_json("/me/messages", &query).await?;

    output::render_rows(w, args.format, &page.value, |e| {
        format!(
            "{received}  {from}  {subject}",
            received = e.received,
            from = if e.from.is_empty() {
                "-".into()
            } else {
                e.from.clone()
            },
            subject = e.subject
        )
    })?;
    Ok(())
}
