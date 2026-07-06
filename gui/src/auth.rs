extern crate alloc;
use alloc::sync::Arc;
use std::sync::{Mutex, PoisonError};

use mailbox_email::{EmailProvider, ImageConnectionError};
use mailbox_shared::{Config, EmailConfig};
use tokio::task::JoinSet;
use tokio::time::error::Elapsed;
use tokio::time::{Duration, timeout};

use crate::GuiApp;

impl GuiApp {
    /// Authenticates with a configuration and gets a new provider.
    ///
    /// # Errors
    ///
    /// Returns a string error giving a vague reason of the failure.
    pub async fn auth(
        email: EmailConfig,
        providers: Arc<Mutex<Vec<EmailProvider>>>,
        config: Arc<Mutex<Config>>,
    ) -> Result<(), &'static str> {
        let provider = Self::auth_one(&email).await?;
        config
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .add_email_config(email)
            .map_err(|_err| "Failed to save configuration")?;
        providers.lock().unwrap_or_else(PoisonError::into_inner).push(provider);
        Ok(())
    }

    /// Authenticate every provider of the config.
    ///
    /// # Errors
    ///
    /// Returns a string error giving a vague reason of the failure.
    #[expect(clippy::iter_over_hash_type, reason = "useless lint")]
    pub async fn auth_config(
        config: Arc<Mutex<Config>>,
        providers: Arc<Mutex<Vec<EmailProvider>>>,
    ) -> Result<(), &'static str> {
        let mut set = {
            let mut set = JoinSet::new();
            let cfg = config.lock().unwrap_or_else(PoisonError::into_inner);
            for email in cfg.as_email_cfgs() {
                let this = email.clone();
                set.spawn(async move { Self::auth_one(&this).await });
            }
            drop(cfg);
            set
        };
        let mut res = Ok(());
        while let Some(next) = set.join_next().await {
            match next {
                Ok(Ok(ok)) => providers
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(ok),
                Ok(Err(err)) => res = Err(err),
                Err(_) => res = Err("Failed to synchronise state"),
            }
        }
        res
    }

    /// Authenticate one provider with the given config.
    async fn auth_one(
        email: &EmailConfig,
    ) -> Result<EmailProvider, &'static str> {
        timeout(Duration::from_mins(1), async {
            EmailProvider::auth(email).await.map_err(|err| match err {
                ImageConnectionError::Login(_) => "Invalid credentials",
                ImageConnectionError::TlsError(_)
                | ImageConnectionError::UnreachableDomain(_)
                | ImageConnectionError::UnreachableDomainThrougnTls(_) =>
                    "Failed to reached specified server",
            })
        })
        .await
        .unwrap_or_else(|_: Elapsed| Err("Failed to connect: timed out"))
    }
}
