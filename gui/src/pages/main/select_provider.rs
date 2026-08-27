extern crate alloc;
use alloc::sync::Arc;
use std::sync::{Mutex, PoisonError};

use iced::widget::Column;
use mailbox_email::EmailProvider;

use crate::ui::component::{btn, txt};
use crate::{Page, Providers};

/// Main page for one provider.
pub struct SelectProviderPage {
    /// Provider currently in use.
    current: Arc<Mutex<EmailProvider>>,
    /// List of active providers.
    list: Providers,
}

impl SelectProviderPage {
    /// Creates a new page.
    pub const fn new(
        current: Arc<Mutex<EmailProvider>>,
        list: Providers,
    ) -> Self {
        Self { current, list }
    }
}

impl Page for SelectProviderPage {
    type Message = SelectProviderMsg;
    type Update = ();

    fn update(&mut self, message: Self::Message) -> Self::Update {
        self.current = message;
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::with_children(
            self.list
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .map(|provider| {
                    let alias = provider
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .alias();
                    btn(txt(alias), Arc::clone(provider)).into()
                }),
        )
        .into()
    }
}

/// Selects a new provider.
pub type SelectProviderMsg = Arc<Mutex<EmailProvider>>;
