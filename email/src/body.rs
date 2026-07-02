use color_eyre::Result;
use mailbox_shared::Message;
use mailparse::{DispositionType, ParsedMail, parse_mail};

/// An attached file.
#[derive(Debug)]
struct Attachment {
    /// Binary content of the file.
    data: Vec<u8>,
    /// Content type of the file.
    mime: String,
    /// Name of the attached file.
    name: String,
}

/// An email body.
#[derive(Debug, Default)]
pub struct EmailBody {
    /// List of files attached to the email.
    attachements: Vec<Attachment>,
    /// Htmm content of the email.
    html: String,
    /// Plain content of the email.
    txt: String,
}

impl EmailBody {
    /// Adds an email part to the current email body.
    fn add(&mut self, part: &ParsedMail<'_>) -> Result<()> {
        let mime = &part.ctype.mimetype;
        let disposition = part.get_content_disposition();
        if disposition.disposition == DispositionType::Attachment {
            self.attachements.push(Attachment {
                name: disposition
                    .params
                    .get("filename")
                    .cloned()
                    .unwrap_or_default(),
                mime: mime.to_owned(),
                data: part.get_body_raw()?,
            });
        } else if mime == "text/plain" && self.txt.is_empty() {
            self.txt = part.get_body()?;
        } else if mime == "text/html" && self.html.is_empty() {
            self.html = part.get_body()?;
        }
        for subpart in &part.subparts {
            self.add(subpart)?;
        }
        Ok(())
    }

    /// Parses an email body from raw bytes.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        let mut this = Self::default();
        this.add(&parse_mail(raw)?)?;
        Ok(this)
    }
}

impl Message for EmailBody {
    fn debug(&self) -> String {
        format!(
            "{}\n==> attachements: {}",
            self.txt,
            self.attachements
                .iter()
                .map(|att| format!("{} ({})", att.name, att.data.len()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
