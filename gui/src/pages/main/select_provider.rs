extern crate alloc;
use alloc::sync::Arc;
use std::sync::Mutex;

use iced::Length;
use iced::widget::{Column, Space, column};
use mailbox_email::EmailProvider;
use mailbox_shared::lock;

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
    type Task = ();
    type Update = Arc<Mutex<EmailProvider>>;

    fn update(&mut self, data: Self::Update) -> Self::Task {
        self.current = data;
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        column!(
            Column::with_children(lock!(self.list).iter().map(|provider| {
                let alias = lock!(provider).alias();
                let msg =
                    SelectProviderMsg::SelectProvider(Arc::clone(provider));
                btn(txt(alias), msg).into()
            }),),
            Space::new().height(Length::Fill),
            btn(txt("+"), SelectProviderMsg::AddProvider)
        )
        .into()
    }
}

/// Message from the provider selector.
#[derive(Clone, Debug)]
pub enum SelectProviderMsg {
    /// Add a new provider.
    AddProvider,
    /// Select a new provider.
    SelectProvider(Arc<Mutex<EmailProvider>>),
}
