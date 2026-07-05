//! GUI mailbox application.
//!
//! The startup pages are successions of loading and checking panels, they are
//! organised as followed:
//!   ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲
//! │ Loading config │
//!   ̅ ̅ ̅ ̅ ̅ ̅ ̅ ̅↓̅ ̅ ̅ ̅ ̅ ̅ ̅ ̅
//!   has a provider?
//!     ↓        ↓
//!     no      yes
//!   ̲ ̲ ̲↓̲ ̲ ̲    ̲ ̲ ̲↓̲ ̲ ̲
//! │ Form │→│ Auth │
//!   ̅ ̅ ̅↑̅ ̅ ̅    ̅ ̅ ̅↓̅ ̅ ̅
//!     no  ← success?
//!              ↓
//!             yes
//!   ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲ ̲↓̲ ̲ ̲ ̲ ̲ ̲ ̲
//! │ Loading (headers) │
//!   ̅ ̅ ̅ ̅ ̲̅ ̲̅ ̲̅ ̲̅↓̲̅ ̲̅ ̲̅ ̲̅ ̲̅ ̲̅ ̅ ̅ ̅ ̅ ̅
//!     │ main app │
//!       ̅ ̅ ̅ ̅ ̅ ̅ ̅ ̅ ̅ ̅
//! .

/// List of pages to display on the screen.
mod pages;

extern crate alloc;
use alloc::sync::Arc;
use std::sync::{Mutex, PoisonError};

use iced::widget::container::Style;
use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task};
use mailbox_email::{EmailProvider, ImageConnectionError};
use mailbox_shared::{Config, EmailConfig, LoadError};
use tokio::time::{Duration, timeout};

use crate::pages::add_config::AddConfigPage;

/// Traits and types required for a page to be rendered and updated.
trait Page {
    /// Messages that are sent after updating the state of the app.
    type Message;
    /// Data passed to the parent in some circumpstances.
    type Update;

    /// Updates the application based on incomming messages.
    fn update(&mut self, message: Self::Message) -> Self::Update;

    /// Displays the app.
    fn view(&self) -> Element<'_, Self::Message>;
}

/// Gui Application state.
#[non_exhaustive]
pub struct GuiApp {
    /// Current page.
    page: GuiAppPage,
    /// List of providers.
    providers: Arc<Mutex<Vec<EmailProvider>>>,
}

/// Gui Application state.
#[non_exhaustive]
pub enum GuiAppPage {
    /// Configuration is empty, open a page to add a provider.
    AddConfig(AddConfigPage),
    /// Configuration is not empty, open default page.
    Main,
}

impl GuiApp {
    /// Authenticates with a configuration and gets a new provider.
    async fn auth(
        config: EmailConfig,
        providers: Arc<Mutex<Vec<EmailProvider>>>,
    ) -> Result<(), &'static str> {
        let provider =
            EmailProvider::auth(&config).await.map_err(|err| match err {
                ImageConnectionError::Login(_) => "Invalid credentials",
                ImageConnectionError::TlsError(_)
                | ImageConnectionError::UnreachableDomain(_)
                | ImageConnectionError::UnreachableDomainThrougnTls(_) =>
                    "Failed to reached specified server",
            })?;
        Config::from(config)
            .save()
            .map_err(|_err| "Failed to save configuration")?;
        providers.lock().unwrap_or_else(PoisonError::into_inner).push(provider);
        Ok(())
    }

    /// Loads the configuration and returns a default [`GuiAppPage`].
    fn new(cfg: &Config) -> Self {
        Self {
            page: if cfg.as_first_email_config().is_none() {
                GuiAppPage::AddConfig(AddConfigPage::default())
            } else {
                GuiAppPage::Main
            },
            providers: Arc::default(),
        }
    }

    /// Runs the gui application.
    ///
    /// # Errors
    ///
    /// Returns an error if the rendering or configuration loading fails.
    pub fn run() -> Result<(), GuiError> {
        let config = Mutex::new(Config::load().map_err(GuiError::Load)?);
        iced::application(
            move || {
                Self::new(
                    &config.lock().unwrap_or_else(PoisonError::into_inner),
                )
            },
            Self::update,
            Self::view,
        )
        .run()
        .map_err(GuiError::Runtime)?;
        Ok(())
    }
}

/// Gui App error.
#[expect(clippy::exhaustive_enums, reason = "same versioning")]
#[derive(Debug)]
pub enum GuiError {
    /// Failed to load initial data before opening the app.
    Load(LoadError),
    /// Failure during runtime.
    ///
    /// The app unexpected panicked.
    Runtime(iced::Error),
}

/// Application messages.
#[derive(Clone, Debug)]
enum Message {
    /// Messages for the add config page.
    AddPage(<AddConfigPage as Page>::Message),
    /// Message for when a provider is added.
    ProviderAdded,
}

impl Page for GuiApp {
    type Message = Message;
    type Update = Task<Self::Message>;

    fn update(&mut self, message: Self::Message) -> Self::Update {
        if let GuiAppPage::AddConfig(page) = &mut self.page {
            page.loading(false);
        }
        match message {
            Message::AddPage(msg) =>
                if let GuiAppPage::AddConfig(page) = &mut self.page
                    && let Some(config) = page.update(msg)
                {
                    page.loading(true);
                    let providers = Arc::clone(&self.providers);
                    return Task::perform(
                        async {
                            timeout(Duration::from_mins(1), async {
                                Self::auth(config, providers).await
                            })
                            .await
                        },
                        |res| match res {
                            Ok(Ok(())) => Message::ProviderAdded,
                            Ok(Err(str)) => Message::AddPage(
                                <AddConfigPage as Page>::Message::Error(str),
                            ),
                            Err(_) => Message::AddPage(
                                <AddConfigPage as Page>::Message::Error(
                                    "Failed to connect: timed out",
                                ),
                            ),
                        },
                    );
                },
            Message::ProviderAdded => self.page = GuiAppPage::Main,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let view = match &self.page {
            GuiAppPage::AddConfig(page) => page.view().map(Message::AddPage),
            GuiAppPage::Main => column!(text("hi"), button("click")).into(),
        };
        container(view)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                ..Default::default()
            })
            .into()
    }
}
