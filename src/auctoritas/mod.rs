//! `auctoritas` — "authority". Authentication and token management.
//!
//! Implements the OAuth2 device authorization grant (RFC 8628) against the
//! Microsoft identity platform, hitting the endpoints directly via `reqwest`
//! rather than pulling in the `oauth2` crate.

pub mod device_code;
pub mod scopes;
pub mod token;

pub use device_code::{DeviceCodeResponse, login_interactive};
pub use token::{KeyringStore, TokenSet, TokenStore};
