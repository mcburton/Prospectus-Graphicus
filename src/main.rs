//! Prospectus Graphicus binary entry point.

use std::io::{self, Write};

use anyhow::Context;
use clap::Parser;

use prospectus_graphicus::cli::{AuthCommand, Cli, Command, MailCommand};
use prospectus_graphicus::commands::{auth, mail};
use prospectus_graphicus::config::{self, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("PROSPECTUS_LOG")
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(io::stderr)
        .init();

    let cli = Cli::parse();

    let cfg_path = match &cli.config {
        Some(p) => p.clone(),
        None => config::default_config_path()?,
    };
    let cfg = Config::load_from(&cfg_path)
        .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;

    let http = reqwest::Client::builder()
        .user_agent(concat!("prospectus-graphicus/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    match cli.command {
        Command::Auth(a) => match a.command {
            AuthCommand::Login => {
                auth::run(
                    &mut stderr,
                    auth::LoginArgs {
                        config: &cfg,
                        http: &http,
                    },
                )
                .await?;
            }
        },
        Command::Mail(m) => match m.command {
            MailCommand::List(args) => {
                mail::run_list(
                    &mut stdout,
                    mail::ListArgs {
                        config: &cfg,
                        http: &http,
                        beta: cli.beta,
                        top: args.top,
                        format: cli.output,
                    },
                )
                .await?;
            }
        },
    }

    stdout.flush().ok();
    Ok(())
}
