//! CLI definition using `clap` derive.

use clap::{Args, Parser, Subcommand};

use crate::output::Format;

#[derive(Debug, Parser)]
#[command(
    name = "prospectus",
    version,
    about = "Prospectus Graphicus — a command-line instrument for handling Outlook's letters through the Microsoft Graph.",
    long_about = "Prospectus Graphicus: Instrumentum Lineae Iussorum ad Epistulas Prospecti per Graphum Tractandas."
)]
pub struct Cli {
    /// Use the Graph beta endpoint instead of v1.0.
    #[arg(long, global = true)]
    pub beta: bool,

    /// Output format (JSON by default; pipes cleanly into `jq`).
    #[arg(long, value_enum, global = true, default_value_t = Format::Json)]
    pub output: Format,

    /// Override the config path (default: `~/.config/prospectus/config.toml`).
    #[arg(long, global = true, env = "PROSPECTUS_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authentication — sign in and manage credentials.
    Auth(AuthArgs),

    /// Mail — `epistulae`.
    Mail(MailArgs),
}

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Sign in via OAuth2 device code flow.
    Login,
}

#[derive(Debug, Args)]
pub struct MailArgs {
    #[command(subcommand)]
    pub command: MailCommand,
}

#[derive(Debug, Subcommand)]
pub enum MailCommand {
    /// List messages from the signed-in user's inbox.
    List(MailListArgs),
}

#[derive(Debug, Args)]
pub struct MailListArgs {
    /// Maximum number of messages to return (Graph `$top`).
    #[arg(long, default_value_t = 25)]
    pub top: u32,
}
