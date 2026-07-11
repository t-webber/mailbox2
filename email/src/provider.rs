extern crate alloc;
use alloc::sync::Arc;

use async_imap::Session;
use async_imap::error::Error as ImapError;
use mailbox_shared::EmailConfig;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

use crate::body::EmailBody;
use crate::header::EmailHeader;
use crate::imap::{
    FetchBodyError, FetchHeadersError, ImageConnectionError, connect_imap, fetch_body, fetch_headers
};

/// Provider for email connections.
#[derive(Debug)]
pub struct EmailProvider {
    /// Alias to use to name the provider.
    alias: char,
    /// Imap session.
    session: Session<TlsStream<TcpStream>>,
}

impl EmailProvider {
    /// Returns the alias of the config.
    #[must_use]
    pub const fn alias(&self) -> char {
        self.alias
    }

    /// Authenticates a configuration into a provider.
    ///
    /// # Errors
    ///
    /// Cf. [`ImageConnectionError`].
    pub async fn auth(
        config: &EmailConfig,
    ) -> Result<Self, ImageConnectionError> {
        Ok(Self { alias: config.alias(), session: connect_imap(config).await? })
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
    ) -> Result<(Vec<EmailHeader>, Vec<ImapError>), FetchHeadersError> {
        fetch_headers(&mut self.session, Arc::from("INBOX")).await
    }
}
