//! Shared traits and functions accross the mailbox applications.

/// Loads and edits config.
mod config;

pub use config::{Config, EmailConfig, LoadError, SaveError};
