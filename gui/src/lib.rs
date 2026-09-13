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
/// Helpers for styling and UI.
mod ui;

extern crate alloc;
use alloc::sync::Arc;
use core::mem::take;
use std::sync::Mutex;

use iced::{Element, Subscription, Task};
use mailbox_email::EmailProvider;
use mailbox_shared::{ArMx, Config, ErrStr, LoadError, errmsg, lock};
use mailbox_whatsapp::Whatsapp;

use crate::pages::{GuiAppMessage, GuiAppPage};

/// Sharable provider.
type Provider = EmailProvider;
/// Sharable list of providers.
type Providers = ArMx<Vec<Provider>>;

/// Traits and types required for a page to be rendered and updated.
trait Page {
    /// Messages that are sent after updating the state of the app.
    type Message;
    /// Data passed to the parent in some circumpstances.
    type Task;
    /// Data passed to the children.
    type Update;

    /// Subscribe to events, like keypresses.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Updates the application based on incomming messages.
    fn update(&mut self, data: Self::Update) -> Self::Task;

    /// Displays the app.
    fn view(&self) -> Element<'_, Self::Message>;
}

/// Gui Application state.
#[non_exhaustive]
pub struct GuiApp {
    /// Configuration.
    config: ArMx<Config>,
    /// Current page.
    page: GuiAppPage,
    /// List of providers.
    providers: Providers,
    /// Whether `WhatsApp` pairing toggle should be available.
    wa_needs_pairing: bool,
}

impl GuiApp {
    /// Displays an error message.
    fn error(&mut self, error: ErrStr) {
        match &mut self.page {
            GuiAppPage::AddConfig(page) => page.error(error),
            GuiAppPage::Authenticate => (),
            GuiAppPage::Main(page) => page.error(error),
        }
    }

    /// Loads the configuration and returns a default [`GuiAppPage`].
    fn new(config: &mut Config) -> (Self, Task<GuiAppMessage>) {
        let has_configs = config.as_first_email_config().is_some();
        let (wa_needs_pairing, wa_failure) = match Whatsapp::new() {
            Ok(wa) => (wa.needs_pairing(), None),
            Err(err) => (false, Some(err)),
        };
        (
            Self {
                page: GuiAppPage::new(has_configs, wa_needs_pairing),
                providers: Arc::default(),
                config: Arc::new(Mutex::new(take(config))),
                wa_needs_pairing,
            },
            wa_failure.map_or_else(
                || {
                    if has_configs {
                        Task::done(GuiAppMessage::Authenticate)
                    } else {
                        Task::none()
                    }
                },
                |err| {
                    Task::done(GuiAppMessage::Error(errmsg!(
                        "WhatsApp initialisation failed",
                        err
                    )))
                },
            ),
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
            move || Self::new(&mut lock!(config)),
            Self::update,
            Self::view,
        )
        .subscription(Self::subscription)
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
