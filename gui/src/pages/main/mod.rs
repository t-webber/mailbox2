/// Lists the headers of the current mailbox.
mod headers;
/// Left bar to select the active provider.
mod select_provider;

extern crate alloc;

use alloc::sync::Arc;
use std::sync::Mutex;

use iced::widget::{Space, container, row};
use iced::{Alignment, Length};
use mailbox_email::EmailProvider;
use mailbox_shared::{ArMx, lock};

pub use crate::pages::main::headers::HeadersMsg;
use crate::pages::main::headers::HeadersPage;
pub use crate::pages::main::select_provider::SelectProviderMsg;
use crate::pages::main::select_provider::SelectProviderPage;
use crate::ui::component::txt;
use crate::{Page, Provider, Providers};

/// Main page for one provider.
pub struct MainPage {
    /// Error to display.
    error: Option<&'static str>,
    /// List of headers.
    headers: HeadersPage,
    /// Whether the app is ready to show data or not.
    loading: ArMx<bool>,
    /// Left bar to select the active provider.
    provider_selector: SelectProviderPage,
}

impl MainPage {
    /// Displays an error message.
    pub const fn error(&mut self, error: &'static str) {
        self.error = Some(error);
    }

    /// Creates a new page.
    pub fn new(current: Arc<Mutex<EmailProvider>>, list: Providers) -> Self {
        Self {
            provider_selector: SelectProviderPage::new(current, list),
            headers: HeadersPage::default(),
            error: None,
            loading: Arc::new(true.into()),
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
        let providers =
            self.provider_selector.view().map(MainMessage::SelectProvider);
        if *lock!(self.loading) {
            row!(
                providers,
                container(
                    txt("loading...")
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                )
            )
        } else {
            row!(
                providers,
                container(self.headers.view().map(MainMessage::Headers))
                    .width(300.),
                Space::new().width(Length::Fill)
            )
        }
        .into()
    }
}

/// Message for the main provider panel.
#[derive(Clone, Debug)]
pub enum MainMessage {
    /// Message from the header list.
    Headers(HeadersMsg),
    /// Message from the provider selector.
    SelectProvider(SelectProviderMsg),
}
