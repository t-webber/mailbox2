//! Mailbox crate.
//!
//! Crate to ease managing a mailbox, including fetching email bodies, checking
//! for new messages not yet pulled, and sending out new email.

#![expect(dead_code, reason = "todo")]

/// Body of the email.
mod body;
/// Handles database connections.
mod db;
/// Structure to handle headers.
mod header;
/// Handles interactions with the IMAP protocol.
mod imap;
/// Implements the provider trait.
mod provider;
/// Decodes the encoded subjects.
mod subject_decoder;
#[cfg(test)]
mod test_subject_decoder;

pub use body::EmailBody;
pub use imap::ImageConnectionError;
pub use provider::EmailProvider;
