use tokio::net::TcpStream;

use async_imap::{Client, Session};
use tokio_native_tls::TlsStream;
use tokio_native_tls::native_tls::TlsConnector;

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
