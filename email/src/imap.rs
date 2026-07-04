extern crate alloc;
use alloc::sync::Arc;
use std::io;

use async_imap::error::Error as ImapError;
use async_imap::{Client, Session};
use mailbox_shared::EmailConfig;
use mailparse::MailParseError;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsStream, native_tls};
use tokio_stream::StreamExt as _;

use crate::body::EmailBody;
use crate::header::Header;

#[allow(
    clippy::allow_attributes,
    clippy::missing_docs_in_private_items,
    missing_docs,
    clippy::exhaustive_enums,
    reason = "err"
)]
#[derive(Debug)]
pub enum ImageConnectionError {
    Login(ImapError),
    TlsError(native_tls::Error),
    UnreachableDomain(io::Error),
    UnreachableDomainThrougnTls(native_tls::Error),
}

#[allow(
    clippy::allow_attributes,
    clippy::missing_docs_in_private_items,
    reason = "err"
)]
#[derive(Debug)]
pub enum FetchBodyError {
    FetchError(ImapError),
    MailboxSelect(ImapError),
    NotFound,
    Parsing(MailParseError),
    Request(ImapError),
}

#[allow(
    clippy::allow_attributes,
    clippy::missing_docs_in_private_items,
    reason = "err"
)]
#[derive(Debug)]
pub enum FetchHeadersError {
    FetchError(ImapError),
    MailboxSelect(ImapError),
    NotFound,
    Request(ImapError),
}

/// Returns an IMAP session.
pub async fn connect_imap(
    cfg: &EmailConfig,
) -> Result<Session<TlsStream<TcpStream>>, ImageConnectionError> {
    let (user, password, domain, port) = cfg.values();
    let tcp = TcpStream::connect((domain, port))
        .await
        .map_err(ImageConnectionError::UnreachableDomain)?;
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(ImageConnectionError::TlsError)?;
    let tls_stream = tokio_native_tls::TlsConnector::from(tls)
        .connect(domain, tcp)
        .await
        .map_err(ImageConnectionError::UnreachableDomainThrougnTls)?;
    Client::new(tls_stream)
        .login(user, password)
        .await
        .map_err(|(err, _unauthenticated_client)| err)
        .map_err(ImageConnectionError::Login)
}

/// Fetches the body of an email, given an inbox and and uid.
pub async fn fetch_body(
    session: &mut Session<TlsStream<TcpStream>>,
    mailbox: &str,
    uid: u32,
) -> Result<EmailBody, FetchBodyError> {
    session.select(mailbox).await.map_err(FetchBodyError::MailboxSelect)?;
    let mut stream = session
        .uid_fetch(uid.to_string(), "BODY.PEEK[]")
        .await
        .map_err(FetchBodyError::Request)?;

    if let Some(message) = stream.next().await
        && let Some(body) = message.map_err(FetchBodyError::FetchError)?.body()
    {
        return EmailBody::parse(body).map_err(FetchBodyError::Parsing);
    }

    Err(FetchBodyError::NotFound)
}

/// Fetches all the headers of all the emails.
pub async fn fetch_headers(
    session: &mut Session<TlsStream<TcpStream>>,
    mailbox: Arc<str>,
) -> Result<(Vec<Header>, Vec<ImapError>), FetchHeadersError> {
    session.select(&mailbox).await.map_err(FetchHeadersError::MailboxSelect)?;

    let mut messages = session
        .fetch("1:*", "(UID ENVELOPE)")
        .await
        .map_err(FetchHeadersError::Request)?;

    let mut headers = vec![];
    let mut errors = vec![];

    while let Some(res_msg) = messages.next().await {
        match res_msg {
            Ok(msg) =>
                if let Some(envelope) = msg.envelope() {
                    headers.push(Header::parse(
                        envelope,
                        Arc::clone(&mailbox),
                        msg.uid.unwrap_or_default(),
                    ));
                },
            Err(err) => errors.push(err),
        }
    }

    Ok((headers, errors))
}
