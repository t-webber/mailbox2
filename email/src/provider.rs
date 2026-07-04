extern crate alloc;
use alloc::sync::Arc;

use async_imap::Session;
use async_imap::error::Error as ImapError;
use mailbox_shared::EmailConfig;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

use crate::body::EmailBody;
use crate::header::Header;
use crate::imap::{
    FetchBodyError, FetchHeadersError, ImageConnectionError, connect_imap, fetch_body, fetch_headers
};

/// Provider for email connections.
pub struct EmailProvider {
    /// Imap session.
    session: Session<TlsStream<TcpStream>>,
}

impl EmailProvider {
    /// Authenticates a configuration into a provider.
    ///
    /// # Errors
    ///
    /// Cf. [`ImageConnectionError`].
    pub async fn auth(
        config: &EmailConfig,
    ) -> Result<Self, ImageConnectionError> {
        Ok(Self { session: connect_imap(config).await? })
    }

    /// Returns the body of an email.
    ///
    /// # Errors
    ///
    /// Cf. [`FetchBodyError`].
    pub async fn get_body(
        &mut self,
        uid: u32,
    ) -> Result<Vec<EmailBody>, FetchBodyError> {
        Ok(vec![fetch_body(&mut self.session, "INBOX", uid).await?])
    }

    /// Returns the list of headers.
    ///
    /// # Errors
    ///
    /// Cf. [`FetchHeadersError`].
    pub async fn get_headers(
        &mut self,
    ) -> Result<(Vec<Header>, Vec<ImapError>), FetchHeadersError> {
        fetch_headers(&mut self.session, Arc::from("INBOX")).await
    }
}
