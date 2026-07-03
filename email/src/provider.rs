extern crate alloc;
use alloc::sync::Arc;

use async_imap::Session;
use color_eyre::Result;
use dotenv::dotenv;
use mailbox_shared::{EmailConfig, Provider};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

use crate::body::EmailBody;
use crate::header::Header;
use crate::imap::{connect_imap, fetch_body, fetch_headers};

/// Provider for email connections.
pub struct EmailProvider {
    /// Imap session.
    session: Session<TlsStream<TcpStream>>,
}

impl Provider for EmailProvider {
    type Auth = EmailConfig;
    type Message = EmailBody;
    type Room = Header;

    async fn auth(config: &EmailConfig) -> Result<Self> {
        dotenv()?;
        Ok(Self { session: connect_imap(config).await? })
    }

    async fn get_messages(
        &mut self,
        room: &Self::Room,
    ) -> Result<Vec<Self::Message>> {
        Ok(vec![fetch_body(&mut self.session, "INBOX", room.uid).await?])
    }

    async fn get_rooms(&mut self) -> Result<Vec<Header>> {
        fetch_headers(&mut self.session, Arc::from("INBOX")).await
    }
}
