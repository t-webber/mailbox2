extern crate alloc;
use alloc::sync::Arc;
use std::env::var;

use async_imap::Session;
use color_eyre::Result;
use dotenv::dotenv;
use mailbox_shared::Provider;
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
    type Message = EmailBody;
    type Room = Header;

    async fn auth() -> Result<Self> {
        dotenv()?;
        Ok(Self {
            session: connect_imap(
                &var("MBX_DOMAIN")?,
                var("MBX_PORT")?.parse()?,
                &var("MBX_USER")?,
                &var("MBX_PASSWORD")?,
            )
            .await?,
        })
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
