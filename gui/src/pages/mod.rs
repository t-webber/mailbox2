/// Page to add a provider configuration.
pub mod add_config;

extern crate alloc;

use alloc::sync::Arc;

use iced::widget::container::Style;
use iced::widget::{button, column, container, text};
use iced::{Color, Element, Length, Task};

use crate::pages::add_config::AddConfigPage;
use crate::{GuiApp, GuiAppPage, Page};

/// Application messages.
#[derive(Clone, Debug)]
pub enum Message {
    /// Messages for the add config page.
    AddPage(<AddConfigPage as Page>::Message),
    /// Authenticates the providers loaded from the configuration.
    Authenticate,
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
                    && let Some(email) = page.update(msg)
                {
                    page.loading(true);
                    let providers = Arc::clone(&self.providers);
                    let config = Arc::clone(&self.config);
                    return Task::perform(
                        Self::auth(email, providers, config),
                        |res| match res {
                            Ok(()) => Message::ProviderAdded,
                            Err(str) => Message::AddPage(
                                <AddConfigPage as Page>::Message::Error(str),
                            ),
                        },
                    );
                },
            Message::ProviderAdded => self.page = GuiAppPage::Main,
            Message::Authenticate => {
                let config = Arc::clone(&self.config);
                let providers = Arc::clone(&self.providers);
                return Task::perform(
                    Self::auth_config(config, providers),
                    |res| match res {
                        Ok(()) => Message::ProviderAdded,
                        Err(str) => Message::AddPage(
                            <AddConfigPage as Page>::Message::Error(str),
                        ),
                    },
                );
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let view = match &self.page {
            GuiAppPage::AddConfig(page) => page.view().map(Message::AddPage),
            GuiAppPage::Main =>
                column!(text("hi").color(Color::WHITE), button("click")).into(),
            GuiAppPage::Authenticate =>
                column!(text("authenticating...").color(Color::WHITE)).into(),
        };
        container(view)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                ..Default::default()
            })
            .into()
    }
}
