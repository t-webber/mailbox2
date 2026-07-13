use iced::Length;
use iced::widget::{Column, Space, column};
use mailbox_shared::lock;

use crate::ui::component::{btn, txt};
use crate::ui::style::{BTN_COLOUR, FOCUSED_COLOUR};
use crate::{Page, Provider, Providers};

/// Main page for one provider.
pub struct SelectProviderPage {
    /// Provider currently in use.
    current: Provider,
    /// List of active providers.
    list: Providers,
}

impl SelectProviderPage {
    /// Returns the current provider.
    pub fn current(&self) -> Provider {
        self.current.clone()
    }

    /// Creates a new page.
    pub const fn new(current: Provider, list: Providers) -> Self {
        Self { current, list }
    }
}

impl Page for SelectProviderPage {
    type Message = SelectProviderMsg;
    type Task = ();
    type Update = Provider;

    fn update(&mut self, data: Self::Update) -> Self::Task {
        self.current = data;
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let current = self.current.alias();
        column!(
            Column::with_children(lock!(self.list).iter().map(|provider| {
                let alias = provider.alias();
                let msg = SelectProviderMsg::SelectProvider(provider.clone());
                btn(
                    txt(alias),
                    msg,
                    current == alias,
                    false,
                    BTN_COLOUR,
                    FOCUSED_COLOUR,
                )
                .into()
            }),),
            Space::new().height(Length::Fill),
            btn(
                txt("+"),
                SelectProviderMsg::AddProvider(self.current.clone()),
                false,
                false,
                BTN_COLOUR,
                FOCUSED_COLOUR
            )
        )
        .into()
    }
}

/// Message from the provider selector.
#[derive(Clone, Debug)]
pub enum SelectProviderMsg {
    /// Add a new provider.
    ///
    /// Also contains the char of the provider to fallback to in case of
    /// cancellation.
    AddProvider(Provider),
    /// Select a new provider.
    SelectProvider(Provider),
}
