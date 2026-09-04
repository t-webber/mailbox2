use alloc::sync::Arc;

use iced::Task;
use mailbox_email::EmailProvider;
use mailbox_shared::{ArMx, Config, EmailConfig, ErrStr, errmsg, lock};
use tokio::task::JoinSet;
use tokio::time::error::Elapsed;
use tokio::time::{Duration, timeout};

use crate::pages::{AddConfigMessage, GuiAppMessage, GuiAppPage, MainPage};
use crate::{GuiApp, Page as _, Provider, Providers};

impl GuiApp {
    /// Adds a new provider configuration.
    pub fn add_config(&mut self, msg: AddConfigMessage) -> Task<GuiAppMessage> {
        if let GuiAppPage::AddConfig(page) = &mut self.page
            && let Some(email) = page.update(msg)
        {
            page.loading(true);
            let providers = Arc::clone(&self.providers);
            let config = Arc::clone(&self.config);
            Task::perform(Self::auth(email, providers, config), |res| match res
            {
                Ok(provider) => GuiAppMessage::ProviderAdded(provider, None),
                Err(str) =>
                    GuiAppMessage::AddConfig(AddConfigMessage::Error(str)),
            })
        } else {
            Task::none()
        }
    }

    /// Adds a new provider.
    pub fn add_provider(
        &mut self,
        current: EmailProvider,
        err: Option<ErrStr>,
    ) -> Task<GuiAppMessage> {
        self.page = GuiAppPage::Main(MainPage::new(
            current,
            Arc::clone(&self.providers),
        ));
        if let Some(str) = err {
            self.error(str);
        }
        if let GuiAppPage::Main(main) = &mut self.page {
            main.boot().map(GuiAppMessage::Main)
        } else {
            Task::none()
        }
    }

    /// Authenticates with a configuration and gets a new provider.
    ///
    /// # Errors
    ///
    /// Returns a string error giving a vague reason of the failure.
    pub async fn auth(
        email: EmailConfig,
        providers: Providers,
        config: ArMx<Config>,
    ) -> Result<Provider, ErrStr> {
        let provider = Self::auth_one(&email).await?;
        lock!(config)
            .add_email_config(email)
            .map_err(|_err| "Failed to save configuration")?;
        lock!(providers).push(provider.clone());
        Ok(provider)
    }

    /// Authenticates the client.
    pub fn auth_and_store(&self) -> Task<GuiAppMessage> {
        let config = Arc::clone(&self.config);
        let providers = Arc::clone(&self.providers);
        Task::perform(Self::auth_config(config, providers), |res| match res {
            (Some(first), err) => GuiAppMessage::ProviderAdded(first, err),
            (None, Some(err)) => GuiAppMessage::Error(err),
            (None, None) => GuiAppMessage::None,
        })
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
    ) -> (Option<Provider>, Option<ErrStr>) {
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
            #[cfg_attr(
                debug_assertions,
                expect(
                    clippy::used_underscore_binding,
                    reason = "used in debug"
                )
            )]
            match next {
                Ok(Ok(ok)) => lock!(providers).push(ok),
                Ok(Err(err)) => res = Some(err),
                Err(_msg) =>
                    res = Some(errmsg!("Failed to synchronise state", _msg)),
            }
        }
        (lock!(providers).first().cloned(), res)
    }

    /// Authenticate one provider with the given config.
    #[cfg_attr(
        debug_assertions,
        expect(clippy::used_underscore_binding, reason = "used in debug")
    )]
    async fn auth_one(email: &EmailConfig) -> Result<EmailProvider, ErrStr> {
        timeout(Duration::from_mins(1), async {
            EmailProvider::auth(email).await.map_err(|err| err.display())
        })
        .await
        .unwrap_or_else(|_msg: Elapsed| {
            Err(errmsg!("Failed to connect: timed out", _msg))
        })
    }
}
