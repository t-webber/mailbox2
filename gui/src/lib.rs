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

use color_eyre::Result;
use iced::widget::{button, column, text};
use iced::{Element, Task};
use mailbox_email::EmailProvider;
use mailbox_shared::{Config, EmailConfig, Provider as _};

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
    ) {
        let provider = EmailProvider::auth(&config).await.unwrap();
        providers.lock().unwrap().push(provider);
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
    pub fn run() -> Result<()> {
        let config = Mutex::new(Config::load()?);
        let boot = move || {
            Self::new(&config.lock().unwrap_or_else(PoisonError::into_inner))
        };
        let app = iced::application(boot, Self::update, Self::view);
        app.run()?;
        Ok(())
    }
}

/// Application messages.
#[derive(Clone)]
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
        match message {
            Message::AddPage(msg) =>
                if let GuiAppPage::AddConfig(page) = &mut self.page
                    && let Some(config) = page.update(msg)
                {
                    return Task::perform(
                        Self::auth(config, Arc::clone(&self.providers)),
                        |()| Message::ProviderAdded,
                    );
                },
            Message::ProviderAdded => self.page = GuiAppPage::Main,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.page {
            GuiAppPage::AddConfig(page) => page.view().map(Message::AddPage),
            GuiAppPage::Main => column!(text("hi"), button("click")).into(),
        }
    }
}
