extern crate alloc;
use alloc::sync::Arc;

use async_imap::{Client, Session};
use color_eyre::Result;
use color_eyre::eyre::bail;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_stream::StreamExt as _;

use crate::body::EmailBody;
use crate::header::Header;

/// Returns an IMAP session.
pub async fn connect_imap(
    domain: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Session<TlsStream<TcpStream>>> {
    let tcp = TcpStream::connect((domain, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls_stream =
        tokio_native_tls::TlsConnector::from(tls).connect(domain, tcp).await?;

    let client = Client::new(tls_stream);

    let session =
        client.login(username, password).await.map_err(|(err, _)| err)?;
    Ok(session)
}

/// Fetches the body of an email, given an inbox and and uid.
pub async fn fetch_body(
    session: &mut Session<TlsStream<TcpStream>>,
    mailbox: &str,
    uid: u32,
) -> Result<EmailBody> {
    session.select(mailbox).await?;

    let mut stream = session.uid_fetch(uid.to_string(), "BODY.PEEK[]").await?;

    if let Some(message) = stream.next().await
        && let Some(body) = message?.body()
    {
        return EmailBody::parse(body);
    }

    bail!("not found")
}

/// Fetches all the headers of all the emails.
pub async fn fetch_headers(
    session: &mut Session<TlsStream<TcpStream>>,
    mailbox: Arc<str>,
) -> Result<Vec<Header>> {
    session.select(&mailbox).await?;

    let mut messages = session.fetch("1:*", "(UID ENVELOPE)").await?;

    let mut headers = vec![];

    while let Some(res_msg) = messages.next().await {
        let msg = res_msg?;
        if let Some(envelope) = msg.envelope() {
            headers.push(Header::parse(
                envelope,
                Arc::clone(&mailbox),
                msg.uid.unwrap_or_default(),
            ));
        }
    }

    Ok(headers)
}
