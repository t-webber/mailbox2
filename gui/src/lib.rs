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

/// Handles authentication.
mod auth;
/// List of pages to display on the screen.
mod pages;

extern crate alloc;
use alloc::sync::Arc;
use core::mem::take;
use std::sync::{Mutex, PoisonError};

use iced::{Element, Task};
use mailbox_email::EmailProvider;
use mailbox_shared::{Config, LoadError};

use crate::pages::Message;
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
    /// Configuration.
    config: Arc<Mutex<Config>>,
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
    /// Authenticate the load configurations.
    Authenticate,
    /// Configuration is not empty, open default page.
    Main,
}

impl GuiApp {
    /// Loads the configuration and returns a default [`GuiAppPage`].
    fn new(config: &mut Config) -> (Self, Task<Message>) {
        let has_configs = config.as_first_email_config().is_some();
        (
            Self {
                page: if has_configs {
                    GuiAppPage::Authenticate
                } else {
                    GuiAppPage::AddConfig(AddConfigPage::default())
                },
                providers: Arc::default(),
                config: Arc::new(Mutex::new(take(config))),
            },
            if has_configs {
                Task::done(Message::Authenticate)
            } else {
                Task::none()
            },
        )
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
                    &mut config.lock().unwrap_or_else(PoisonError::into_inner),
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
