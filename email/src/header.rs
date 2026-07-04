extern crate alloc;

use alloc::borrow::Cow;
use alloc::sync::Arc;

use async_imap::imap_proto::{Address, Envelope};

use crate::subject_decoder::decode_subject;

/// Helper to access a field of an envelope.
macro_rules! field {
    ($field:expr) => {
        $field.as_deref().map(String::from_utf8_lossy)
    };
}

/// Helper to display a lines for the debug method.
macro_rules! item {
    ($lines:ident $val:expr) => {
        if !$val.is_empty() {
            $lines.push(format!(
                concat!(stringify!($val), ": {}"),
                $val.join(", ")
            ))
        }
    };
}

/// IMAP header.
#[derive(Debug)]
pub struct Header {
    /// Blind carbon copy.
    pub bcc: Vec<String>,
    /// Carbon copy.
    pub cc: Vec<String>,
    /// Sent date.
    pub date: Option<String>,
    /// User received from.
    pub from: Vec<String>,
    /// Thread conversation email.
    pub in_reply_to: Option<String>,
    /// Name of the mailbox corresponding to the unique id.
    pub mailbox: Arc<str>,
    /// Immutable message id.
    pub message_id: String,
    /// To whom to reply.
    pub reply_to: Vec<String>,
    /// Mailing-list received from.
    pub sender: Vec<String>,
    /// Decoded subject.
    pub subject: String,
    /// To whom it was sent.
    pub to: Vec<String>,
    /// Unique email id.
    pub uid: u32,
}

impl Header {
    /// Parses a header from it's envelope.
    pub fn parse(envelope: &Envelope<'_>, mailbox: Arc<str>, uid: u32) -> Self {
        Self {
            mailbox,
            from: serialises_addresses(envelope.from.as_ref()),
            subject: field!(envelope.subject).map_or_else(
                || "<no subject>".to_owned(),
                |subject| decode_subject(&subject),
            ),
            uid,
            bcc: serialises_addresses(envelope.bcc.as_ref()),
            cc: serialises_addresses(envelope.cc.as_ref()),
            date: field!(envelope.date).map(Cow::into_owned),
            in_reply_to: field!(envelope.in_reply_to).map(Cow::into_owned),
            reply_to: serialises_addresses(envelope.reply_to.as_ref()),
            sender: serialises_addresses(envelope.sender.as_ref()),
            to: serialises_addresses(envelope.to.as_ref()),
            message_id: field!(envelope.message_id)
                .unwrap_or_default()
                .into_owned(),
        }
    }
}

impl Header {
    /// Pretty-print for cli usage.
    pub fn debug(&self) -> String {
        let Self {
            bcc,
            cc,
            mailbox,
            date,
            from,
            in_reply_to,
            message_id,
            reply_to,
            sender,
            subject,
            to,
            uid,
        } = self;
        let mut lines = vec![
            format!("subject: {subject}"),
            format!("mid: {message_id}"),
            format!("uid: {uid}"),
            format!("box: {mailbox}"),
        ];
        item!(lines from);
        item!(lines sender);
        item!(lines to);
        item!(lines cc);
        item!(lines bcc);
        item!(lines reply_to);
        if let Some(inner) = date {
            lines.push(format!("date: {inner}"));
        }
        if let Some(inner) = in_reply_to {
            lines.push(format!("in_reply_to: {inner}"));
        }
        lines.join("\n")
    }

    /// Returns the list of senders.
    pub fn from(&self) -> String {
        self.from.join(", ")
    }

    /// Returns the subject.
    pub const fn subject(&self) -> &str {
        self.subject.as_str()
    }
}

/// Converts a list of addresses to a list of strings.
fn serialises_addresses(addrs: Option<&Vec<Address<'_>>>) -> Vec<String> {
    addrs
        .as_ref()
        .map(|inner| {
            inner.iter().map(serialise_address).collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

/// Converts an address to a string.
fn serialise_address(addr: &Address<'_>) -> String {
    {
        let mailbox = field!(addr.mailbox).unwrap_or_default();
        let host = field!(addr.host)
            .map(|host| format!("@{host}"))
            .unwrap_or_default();
        field!(addr.name).map_or_else(
            || {
                let addr_str = format!("{mailbox}{host}");
                if addr_str.is_empty() {
                    "<no sender>".to_owned()
                } else {
                    addr_str
                }
            },
            |name| format!("{} <{mailbox}{host}>", decode_subject(&name)),
        )
    }
}
