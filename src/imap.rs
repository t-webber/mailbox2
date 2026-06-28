use async_imap::{Client, Session};
use color_eyre::Result;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_stream::StreamExt;

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
            let subject = envelope
                .subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_else(|| "<no subject>".to_string());

            let from = envelope
                .from
                .as_ref()
                .and_then(|f| f.first())
                .and_then(|addr| {
                    let mailbox = addr
                        .mailbox
                        .as_ref()
                        .map(|m| String::from_utf8_lossy(m).to_string());
                    let host = addr
                        .host
                        .as_ref()
                        .map(|h| String::from_utf8_lossy(h).to_string());

                    match (mailbox, host) {
                        (Some(m), Some(h)) => Some(format!("{m}@{h}")),
                        _ => None,
                    }
                })
                .unwrap_or_else(|| "<unknown>".to_string());

            println!("From: {from}");
            println!("Subject: {subject}");
            println!("---");
        }
    }

    Ok(())
}
