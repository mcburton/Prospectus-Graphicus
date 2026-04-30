//! OAuth2 scopes requested by Prospectus Graphicus.
//!
//! Requested up-front so future command slices (send, folders, etc.) don't
//! trigger a re-consent prompt.

/// Scopes requested for all user-delegated flows.
pub const DEFAULT_SCOPES: &[&str] = &[
    "offline_access",
    "User.Read",
    "Mail.Read",
    "Mail.ReadWrite",
    "Mail.Send",
];

pub fn joined() -> String {
    DEFAULT_SCOPES.join(" ")
}
