//! Mailbox crate.
//!
//! Crate to ease managing a mailbox, including fetching email bodies, checking
//! for new messages not yet pulled, and sending out new email.

#![expect(dead_code, reason = "todo")]

/// Body of the email.
mod body;
/// Structure to handle headers.
mod header;
/// Implements the provider trait.
mod provider;
/// Decodes the encoded subjects.
mod subject_decoder;
#[cfg(test)]
mod test_subject_decoder;

extern crate alloc;
pub use body::EmailBody;
pub use header::EmailHeader;
pub use provider::{
    EmailProvider, FetchBodyError, FetchHeadersError, ImapConnectionError, ListBoxError, SelectBoxError
};
