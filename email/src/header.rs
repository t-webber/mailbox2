use alloc::borrow::Cow;
use core::cmp::Ordering;

use async_imap::imap_proto::{Address, Envelope};
use chrono::{DateTime, Datelike as _, FixedOffset, Timelike as _};

use crate::subject_decoder::decode_subject;

/// Helper to access a field of an envelope.
macro_rules! field {
    ($field:expr) => {
        $field.as_deref().map(String::from_utf8_lossy)
    };
}

/// IMAP header.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct EmailHeader {
    /// Blind carbon copy.
    pub bcc: Vec<String>,
    /// Carbon copy.
    pub cc: Vec<String>,
    /// Sent date.
    pub date: Option<DateTime<FixedOffset>>,
    /// User received from.
    pub from: Vec<String>,
    /// Thread conversation email.
    pub in_reply_to: Option<String>,
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

impl Ord for EmailHeader {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.date, other.date) {
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(this), Some(that)) => that.cmp(&this),
        }
    }
}

impl PartialOrd for EmailHeader {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl EmailHeader {
    /// Parses a header from it's envelope.
    pub fn parse(envelope: &Envelope<'_>, uid: u32) -> Self {
        Self {
            from: serialises_addresses(envelope.from.as_ref()),
            subject: field!(envelope.subject).map_or_else(
                || "<no subject>".to_owned(),
                |subject| decode_subject(&subject),
            ),
            uid,
            bcc: serialises_addresses(envelope.bcc.as_ref()),
            cc: serialises_addresses(envelope.cc.as_ref()),
            date: if let Some(raw_date) = field!(envelope.date)
                && let Ok(date) =
                    DateTime::parse_from_rfc2822(raw_date.as_ref())
            {
                Some(date)
            } else {
                None
            },
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

impl EmailHeader {
    /// Returns the sent date, if present.
    #[must_use]
    pub fn date(&self) -> String {
        self.date.map_or_default(|dt| {
            format!(
                "{:02}/{:02}/{:02} {:02}:{:02}",
                dt.day(),
                dt.month(),
                dt.year().rem_euclid(2000i32),
                dt.hour(),
                dt.minute()
            )
        })
    }

    /// Returns the list of senders.
    #[must_use]
    pub fn from(&self) -> String {
        self.from.join(", ")
    }

    /// Returns the subject.
    #[must_use]
    pub const fn subject(&self) -> &str {
        self.subject.as_str()
    }
}

/// Converts a list of addresses to a list of strings.
fn serialises_addresses(addrs: Option<&Vec<Address<'_>>>) -> Vec<String> {
    addrs.as_ref().map_or_default(|inner| {
        inner.iter().map(serialise_address).collect::<Vec<String>>()
    })
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
