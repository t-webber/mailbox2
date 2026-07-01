extern crate alloc;
use alloc::borrow::Cow;

use async_imap::imap_proto::Address;
use async_imap::{Client, Session};
use color_eyre::Result;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_stream::StreamExt as _;

use crate::header::Header;
use crate::subject_decoder::decode_subject;

/// Helper to access a field of an envelope.
macro_rules! field {
    ($field:expr) => {
        $field.as_deref().map(String::from_utf8_lossy)
    };
}

/// Returns an IMAP session.
pub async fn connect_imap(
    domain: &str,
    port: u16,
    username: &str,
    password: &str,
) -> color_eyre::Result<Session<TlsStream<TcpStream>>> {
    let tcp = TcpStream::connect((domain, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls_stream =
        tokio_native_tls::TlsConnector::from(tls).connect(domain, tcp).await?;

    let client = Client::new(tls_stream);

    let session =
        client.login(username, password).await.map_err(|(err, _)| err)?;
    Ok(session)
}

/// Fetches all the headers of all the emails.
pub async fn fetch_headers(
    session: &mut Session<TlsStream<TcpStream>>,
) -> Result<Vec<Header>> {
    session.select("INBOX").await?;

    let mut messages = session.fetch("1:*", "(UID ENVELOPE)").await?;

    let mut headers = vec![];

    while let Some(res_msg) = messages.next().await {
        let msg = res_msg?;
        if let Some(envelope) = msg.envelope() {
            let subject = field!(envelope.subject).map_or_else(
                || "<no subject>".to_owned(),
                |subject| decode_subject(&subject),
            );

            headers.push(Header {
                from: serialises_addresses(envelope.from.as_ref()),
                subject,
                uid: msg.uid.unwrap_or_default(),
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
            });
        }
    }

    Ok(headers)
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
