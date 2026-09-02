use alloc::sync::Arc;
use std::collections::HashSet;
use std::io;

use async_imap::error::Error as ImapError;
use async_imap::{Client, Session};
use mailbox_shared::{EmailConfig, TokioMutex, error, log};
use mailparse::MailParseError;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_native_tls::{TlsStream, native_tls};
use tokio_stream::StreamExt as _;
use utf7_imap::{decode_utf7_imap, encode_utf7_imap};

use crate::body::EmailBody;
use crate::header::EmailHeader;

/// Connected IMAP handle to do requests.
#[derive(Debug)]
struct ImapSession(Session<TlsStream<TcpStream>>);

impl ImapSession {
    /// Authenticates a configuration into a provider.
    ///
    /// # Errors
    ///
    /// Cf. [`ImapConnectionError`].
    pub async fn auth(
        config: &EmailConfig,
    ) -> Result<Self, ImapConnectionError> {
        log!("Authenticating {}", config.alias());
        let (user, password, domain, port) = config.values();
        log!("> tcp connect");
        let tcp = TcpStream::connect((domain, port))
            .await
            .map_err(ImapConnectionError::UnreachableDomain)?;
        log!("> tls build");
        let tls = native_tls::TlsConnector::builder()
            .build()
            .map_err(ImapConnectionError::TlsError)?;
        log!("> tls connect");
        let tls_stream = tokio_native_tls::TlsConnector::from(tls)
            .connect(domain, tcp)
            .await
            .map_err(ImapConnectionError::UnreachableDomainThrougnTls)?;
        log!("> client connect");
        let session = Client::new(tls_stream)
            .login(user, password)
            .await
            .map_err(|(err, _unauthenticated_client)| err)
            .map_err(ImapConnectionError::Login)?;
        log!("Authenticated {}", config.alias());
        Ok(Self(session))
    }
}

/// Provider for email connections.
#[derive(Debug, Clone)]
pub struct EmailProvider {
    /// Alias to use to name the provider.
    alias: char,
    /// Imap session.
    background_session: Arc<TokioMutex<ImapSession>>,
    /// Foreground session to execute priority work.
    priority_session: Arc<TokioMutex<ImapSession>>,
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
    /// Cf. [`ImapConnectionError`].
    pub async fn auth(
        config: &EmailConfig,
    ) -> Result<Self, ImapConnectionError> {
        Ok(Self {
            alias: config.alias(),
            background_session: Arc::new(TokioMutex::new(
                ImapSession::auth(config).await?,
            )),
            priority_session: Arc::new(TokioMutex::new(
                ImapSession::auth(config).await?,
            )),
        })
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
        log!("Fetching body of {uid}");
        let mut session = self.priority_session.lock().await;
        let mut stream = session
            .0
            .uid_fetch(uid.to_string(), "BODY.PEEK[]")
            .await
            .map_err(FetchBodyError::Request)?;
        if let Some(message) = stream.next().await
            && let Some(body) =
                message.map_err(FetchBodyError::FetchError)?.body()
        {
            log!("Fetched body of {uid}");
            return EmailBody::parse(body).map_err(FetchBodyError::Parsing);
        }
        drop(stream);
        drop(session);
        log!("Body of {uid} not found");
        Err(FetchBodyError::NotFound(()))
    }

    /// Returns the list of headers.
    ///
    /// # Errors
    ///
    /// Cf. [`FetchHeadersError`].
    #[must_use]
    pub fn get_headers(
        &self,
    ) -> mpsc::Receiver<Result<Arc<EmailHeader>, FetchHeadersError>> {
        log!("Fetching headers");
        let (tx, rx) = mpsc::channel(32);
        let async_session = Arc::clone(&self.background_session);

        tokio::spawn(async move {
            let mut session = async_session.lock().await;
            let mut messages = match session
                .0
                .fetch("1:*", "(UID ENVELOPE)")
                .await
                .map_err(FetchHeadersError::Request)
            {
                Ok(messages) => messages,
                Err(res) => {
                    #[expect(
                        clippy::used_underscore_binding,
                        reason = "used in debug"
                    )]
                    if let Err(_err) = tx.send(Err(res)).await {
                        log!("Fetching headers bailed: {_err}");
                    }
                    return;
                }
            };

            while let Some(res) = messages.next().await {
                #[expect(
                    clippy::used_underscore_binding,
                    reason = "used in debug"
                )]
                if let Err(_err) = tx
                    .send(match res {
                        Ok(msg) =>
                            if let Some(envelope) = msg.envelope() {
                                Ok(Arc::new(EmailHeader::parse(
                                    envelope,
                                    msg.uid.unwrap_or_default(),
                                )))
                            } else {
                                Err(FetchHeadersError::NoHeader(()))
                            },
                        Err(err) => Err(FetchHeadersError::Request(err)),
                    })
                    .await
                {
                    log!("Fetching headers bailed: {_err}");
                }
            }
            drop(messages);
            drop(session);
        });

        rx
    }

    /// List mailboxes.
    ///
    /// # Errors
    ///
    /// Cf. [`ListBoxError`].
    pub async fn get_mailboxes(
        &self,
    ) -> Result<(Vec<Arc<str>>, Vec<ListBoxError>), ListBoxError> {
        log!("Fetching mailboxes");
        let mut session = self.background_session.lock().await;
        let mut res = vec![];
        let mut errors = vec![];
        let mut mailboxes = session
            .0
            .list(None, Some("*"))
            .await
            .map_err(ListBoxError::Request)?;
        while let Some(next) = mailboxes.next().await {
            match next {
                Ok(mailbox) => res.push(Arc::from(decode_utf7_imap(
                    mailbox.name().to_owned(),
                ))),
                Err(err) => errors.push(ListBoxError::Request(err)),
            }
        }
        drop(mailboxes);
        drop(session);
        log!("Fetched {} mailboxes", res.len());
        Ok((res, errors))
    }

    /// List unseen emails.
    ///
    /// # Errors
    ///
    /// Cf. [`UnseenError`].
    pub async fn get_unseen(&self) -> Result<HashSet<u32>, UnseenError> {
        log!("Fetch unseen emails");
        let mut session = self.background_session.lock().await;
        let uids = session
            .0
            .uid_search("UNSEEN")
            .await
            .map_err(UnseenError::Request)?;
        drop(session);
        log!("Fetch {} unseen emails", uids.len());
        Ok(uids)
    }

    /// Select a different mailbox.
    ///
    /// # Errors
    ///
    /// Cf. [`SelectBoxError`].
    pub async fn select_mailbox(
        &self,
        name: String,
    ) -> Result<(), SelectBoxError> {
        let encoded_name = encode_utf7_imap(name);
        self.priority_session
            .lock()
            .await
            .0
            .select(&encoded_name)
            .await
            .map_err(SelectBoxError::Request)?;
        self.background_session
            .lock()
            .await
            .0
            .select(&encoded_name)
            .await
            .map_err(SelectBoxError::Request)?;
        Ok(())
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
    Request ImapError: "Connection error",
    Channel mpsc::error::SendError<()>: "Failed to send header through channel",
    NoHeader (): "Failed to fetch email header",
);

error!(UnseenError:
    Request ImapError: "Connection error",
);

error!(ListBoxError:
    Request ImapError: "Connection error",
);

error!(SelectBoxError:
    Request ImapError: "Failed to select mailbox",
);
