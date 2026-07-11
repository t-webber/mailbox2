/// Left bar to select the active provider.
mod select_provider;

extern crate alloc;

use alloc::sync::Arc;
use std::sync::Mutex;

use mailbox_email::EmailProvider;

use crate::pages::main::select_provider::{
    SelectProviderMsg, SelectProviderPage
};
use crate::{Page, Provider, Providers};

/// Main page for one provider.
pub struct MainPage {
    /// Error to display.
    error: Option<&'static str>,
    /// Left bar to select the active provider.
    provider_selector: SelectProviderPage,
}

impl MainPage {
    /// Displays an error message.
    pub const fn error(&mut self, error: &'static str) {
        self.error = Some(error);
    }

    /// Creates a new page.
    pub const fn new(
        current: Arc<Mutex<EmailProvider>>,
        list: Providers,
    ) -> Self {
        Self {
            provider_selector: SelectProviderPage::new(current, list),
            error: None,
        }
    }
}

impl Page for MainPage {
    type Message = MainMessage;
    type Task = ();
    type Update = Provider;

    fn update(&mut self, data: Self::Update) -> Self::Task {
        self.provider_selector.update(data);
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        self.provider_selector.view()
    }
}

/// Message for the main provider panel.
pub type MainMessage = SelectProviderMsg;
