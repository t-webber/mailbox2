//! Shared traits and functions accross the mailbox applications.

/// Loads and edits config.
mod config;
/// Provider traits and methods to easily replace one provider with another.
mod provider;

pub use config::{Config, EmailConfig};
pub use provider::{Message, Provider, Room};
