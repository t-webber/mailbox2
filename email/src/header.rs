extern crate alloc;

use alloc::borrow::Cow;
use std::sync::Arc;

use mailbox_shared::Room;

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

impl Room for Header {
    fn debug(&self) -> String {
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

    fn name(&self) -> Cow<'_, str> {
        self.from.join(", ").into()
    }

    fn overview(&self) -> Cow<'_, str> {
        self.subject.as_str().into()
    }
}
