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
