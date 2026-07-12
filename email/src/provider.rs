extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use std::collections::HashSet;
use std::io;

use async_imap::error::Error as ImapError;
use async_imap::{Client, Session};
use mailbox_shared::{ArMx, EmailConfig, StdMutex, TokioMutex, error};
use mailparse::MailParseError;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsStream, native_tls};
use tokio_stream::StreamExt as _;

use crate::body::EmailBody;
use crate::header::EmailHeader;

/// Provider for email connections.
#[derive(Debug, Clone)]
pub struct EmailProvider {
    /// Alias to use to name the provider.
    alias: char,
    /// Imap session.
    session: Arc<TokioMutex<Session<TlsStream<TcpStream>>>>,
}

impl EmailProvider {
    /// Returns the alias of the provider.
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
    ) -> Result<Self, ImapConnectionError> {
        let (user, password, domain, port) = config.values();
        let tcp = TcpStream::connect((domain, port))
            .await
            .map_err(ImapConnectionError::UnreachableDomain)?;
        let tls = native_tls::TlsConnector::builder()
            .build()
            .map_err(ImapConnectionError::TlsError)?;
        let tls_stream = tokio_native_tls::TlsConnector::from(tls)
            .connect(domain, tcp)
            .await
            .map_err(ImapConnectionError::UnreachableDomainThrougnTls)?;
        let session = Client::new(tls_stream)
            .login(user, password)
            .await
            .map_err(|(err, _unauthenticated_client)| err)
            .map_err(ImapConnectionError::Login)?;
        Ok(Self { alias: config.alias(), session: Arc::new(session.into()) })
    }

    /// Returns the body of an email.
    ///
    /// # Errors
    ///
    /// Cf. [`FetchBodyError`].
    pub async fn get_body(
        &self,
        uid: u32,
    ) -> Result<EmailBody, FetchBodyError> {
        let mut session = self.session.lock().await;
        session.select("INBOX").await.map_err(FetchBodyError::MailboxSelect)?;
        let mut stream = session
            .uid_fetch(uid.to_string(), "BODY.PEEK[]")
            .await
            .map_err(FetchBodyError::Request)?;
        if let Some(message) = stream.next().await
            && let Some(body) =
                message.map_err(FetchBodyError::FetchError)?.body()
        {
            return EmailBody::parse(body).map_err(FetchBodyError::Parsing);
        }
        drop(stream);
        drop(session);
        Err(FetchBodyError::NotFound(()))
    }

    /// Returns the list of headers.
    ///
    /// # Errors
    ///
    /// Cf. [`FetchHeadersError`].
    pub async fn get_headers(
        &self,
    ) -> Result<
        (Vec<ArMx<EmailHeader>>, Vec<FetchHeadersError>),
        FetchHeadersError,
    > {
        let mut session = self.session.lock().await;
        let mailbox = Arc::from("INBOX");
        session
            .select(&mailbox)
            .await
            .map_err(FetchHeadersError::MailboxSelect)?;
        let mut messages = session
            .fetch("1:*", "(UID ENVELOPE)")
            .await
            .map_err(FetchHeadersError::Request)?;
        let mut headers = BTreeSet::new();
        let mut errors = vec![];
        while let Some(res_msg) = messages.next().await {
            match res_msg {
                Ok(msg) =>
                    if let Some(envelope) = msg.envelope() {
                        headers.insert(EmailHeader::parse(
                            envelope,
                            Arc::clone(&mailbox),
                            msg.uid.unwrap_or_default(),
                        ));
                    },
                Err(err) => errors.push(FetchHeadersError::Request(err)),
            }
        }
        drop(messages);
        drop(session);
        Ok((
            headers
                .into_iter()
                .rev()
                .map(|header| Arc::new(StdMutex::new(header)))
                .collect(),
            errors,
        ))
    }

    /// List unseen emails.
    ///
    /// # Errors
    ///
    /// Cf. [`UnseenError`].
    pub async fn get_unseen(&self) -> Result<HashSet<u32>, UnseenError> {
        let mut session = self.session.lock().await;
        let mailbox = Arc::from("INBOX");
        session.select(&mailbox).await.map_err(UnseenError::MailboxSelect)?;
        let uids =
            session.uid_search("UNSEEN").await.map_err(UnseenError::Request)?;
        drop(session);
        Ok(uids)
    }
}

error!(ImapConnectionError:
    Login ImapError: "Invalid credentials",
    TlsError native_tls::Error: "TLS error",
    UnreachableDomainThrougnTls native_tls::Error: "Domain unreachable through TLS",
    UnreachableDomain io::Error: "Domain unreachable",
);

error!(FetchBodyError:
    FetchError ImapError: "Failed to fetch headers",
    MailboxSelect ImapError: "Failed to select mailbox",
    NotFound (): "This email was deleted",
    Parsing MailParseError: "Invalid email body format",
    Request ImapError: "Connection error",
);

error!(FetchHeadersError:
    MailboxSelect ImapError: "Failed to select mailbox",
    Request ImapError: "Connection error",
);

error!(UnseenError:
    MailboxSelect ImapError: "Failed to select mailbox",
    Request ImapError: "Connection error",
);
