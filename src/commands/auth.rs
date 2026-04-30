//! `prospectus auth login`: acquire a refresh token via device code flow.

use std::io::Write;

use crate::auctoritas::{KeyringStore, TokenStore, device_code::login_interactive};
use crate::config::Config;
use crate::error::Result;

pub struct LoginArgs<'a> {
    pub config: &'a Config,
    pub http: &'a reqwest::Client,
}

/// Run the interactive login, writing progress messages to `stderr`.
pub async fn run<W: Write>(stderr: &mut W, args: LoginArgs<'_>) -> Result<()> {
    let store = KeyringStore::new()?;
    run_with_store(stderr, args, &store).await
}

/// Testable variant that accepts a `TokenStore` implementation.
pub async fn run_with_store<W: Write>(
    stderr: &mut W,
    args: LoginArgs<'_>,
    store: &dyn TokenStore,
) -> Result<()> {
    let _token = login_interactive(args.http, args.config, store, |device| {
        if let Some(msg) = &device.message {
            let _ = writeln!(stderr, "{msg}");
        } else {
            let _ = writeln!(
                stderr,
                "To sign in, visit {} and enter code {}",
                device.verification_uri, device.user_code
            );
        }
        let _ = writeln!(stderr, "Waiting for authorization...");
    })
    .await?;

    writeln!(
        stderr,
        "Login successful. Refresh token stored in system keyring."
    )?;
    Ok(())
}
