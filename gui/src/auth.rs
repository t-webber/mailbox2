use mailbox_email::EmailProvider;
use mailbox_shared::{ArMx, Config, EmailConfig, lock};
use tokio::task::JoinSet;
use tokio::time::error::Elapsed;
use tokio::time::{Duration, timeout};

use crate::{GuiApp, Provider, Providers};

impl GuiApp {
    /// Authenticates with a configuration and gets a new provider.
    ///
    /// # Errors
    ///
    /// Returns a string error giving a vague reason of the failure.
    pub async fn auth(
        email: EmailConfig,
        providers: Providers,
        config: ArMx<Config>,
    ) -> Result<Provider, &'static str> {
        let provider = Self::auth_one(&email).await?;
        lock!(config)
            .add_email_config(email)
            .map_err(|_err| "Failed to save configuration")?;
        lock!(providers).push(provider.clone());
        Ok(provider)
    }

    /// Authenticate every provider of the config.
    ///
    /// # Errors
    ///
    /// Returns a string error giving a vague reason of the failure.
    #[expect(clippy::iter_over_hash_type, reason = "useless lint")]
    pub async fn auth_config(
        config: ArMx<Config>,
        providers: Providers,
    ) -> (Option<Provider>, Option<&'static str>) {
        let mut set = {
            let mut set = JoinSet::new();
            for email in lock!(config).as_email_cfgs() {
                let this = email.clone();
                set.spawn(async move { Self::auth_one(&this).await });
            }
            set
        };
        let mut res = None;
        while let Some(next) = set.join_next().await {
            match next {
                Ok(Ok(ok)) => lock!(providers).push(ok),
                Ok(Err(err)) => res = Some(err),
                Err(_) => res = Some("Failed to synchronise state"),
            }
        }
        (lock!(providers).first().cloned(), res)
    }

    /// Authenticate one provider with the given config.
    async fn auth_one(
        email: &EmailConfig,
    ) -> Result<EmailProvider, &'static str> {
        timeout(Duration::from_mins(1), async {
            EmailProvider::auth(email).await.map_err(|err| err.display())
        })
        .await
        .unwrap_or_else(|_: Elapsed| Err("Failed to connect: timed out"))
    }
}
