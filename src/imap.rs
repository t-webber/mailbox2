use async_imap::{Client, Session};
use color_eyre::Result;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_stream::StreamExt;

use crate::subject_decoder::decode_subject;

pub async fn connect_imap(
    domain: &str,
    port: u16,
    username: &str,
    password: &str,
) -> color_eyre::Result<Session<TlsStream<TcpStream>>> {
    let tcp = TcpStream::connect((domain, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls_stream = tokio_native_tls::TlsConnector::from(tls)
        .connect(domain, tcp)
        .await?;

    let client = Client::new(tls_stream);

    let session = client.login(username, password).await.map_err(|(e, _)| e)?;
    Ok(session)
}

pub async fn fetch_headers(session: &mut Session<TlsStream<TcpStream>>) -> Result<()> {
    session.select("INBOX").await?;

    let mut messages = session.fetch("1:*", "(ENVELOPE)").await?;

    while let Some(msg) = messages.next().await {
        let msg = msg?;

        if let Some(envelope) = msg.envelope() {
            let subject = envelope.subject.as_deref().map_or_else(
                || Ok::<_, color_eyre::Report>("<no subject>".to_owned()),
                |subject| Ok(decode_subject(str::from_utf8(subject)?)),
            )?;

            let from = envelope
                .from
                .as_ref()
                .and_then(|f| f.first())
                .and_then(|addr| {
                    let mailbox = addr
                        .mailbox
                        .as_deref()
                        .map(|s| String::from_utf8_lossy(s))?;
                    let host = addr.host.as_deref().map(String::from_utf8_lossy)?;
                    Some(format!("{mailbox}@{host}"))
                })
                .unwrap_or_else(|| "<no sender>".to_owned());

            println!("From: {from}");
            println!("Subject: {subject}");
            println!("---");
        }
    }

    Ok(())
}
