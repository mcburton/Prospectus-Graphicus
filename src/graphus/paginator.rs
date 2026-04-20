//! `@odata.nextLink` iteration helper. Currently unused — wired for the
//! upcoming `--all` flag on listing commands.

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::Result;

use super::GraphClient;

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    pub value: Vec<T>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

/// Fetch every page by following `@odata.nextLink`. Stops when the link is
/// absent.
pub async fn collect_all<T>(client: &GraphClient, first_path: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let mut out = Vec::new();
    let mut page: Page<T> = client.get_json(first_path, &[("", "")][..0]).await?;
    out.extend(page.value);
    while let Some(link) = page.next_link.take() {
        page = client.get_url(&link).await?;
        out.extend(page.value);
    }
    Ok(out)
}
