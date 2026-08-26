/// Page to add a provider configuration.
mod add_config;
/// Page provider page after authentication.
mod main;

extern crate alloc;

use alloc::sync::Arc;

use iced::widget::container::Style;
use iced::widget::{column, container, text};
use iced::{Color, Element, Length, Task};

use crate::pages::add_config::{AddConfigMessage, AddConfigPage};
use crate::pages::main::MainPage;
use crate::{GuiApp, Page};

/// Application messages.
#[derive(Clone, Debug)]
pub enum GuiAppMessage {
    /// Messages for the add config page.
    AddConfig(AddConfigMessage),
    /// Authenticates the providers loaded from the configuration.
    Authenticate,
    /// Message for the main page.
    Main(()),
    /// Message for when a provider is added.
    ProviderAdded,
}

/// Gui Application state.
#[non_exhaustive]
pub enum GuiAppPage {
    /// Configuration is empty, open a page to add a provider.
    AddConfig(AddConfigPage),
    /// Authenticate the load configurations.
    Authenticate,
    /// Configuration is not empty, open default page.
    Main(MainPage),
}

impl GuiAppPage {
    /// Opens the first page depending on whether configs where found or not.
    pub fn new(has_configs: bool) -> Self {
        if has_configs {
            Self::Authenticate
        } else {
            Self::AddConfig(AddConfigPage::default())
        }
    }
}

impl Page for GuiApp {
    type Message = GuiAppMessage;
    type Update = Task<Self::Message>;

    fn update(&mut self, message: Self::Message) -> Self::Update {
        if let GuiAppPage::AddConfig(page) = &mut self.page {
            page.loading(false);
        }
        match message {
            GuiAppMessage::AddConfig(msg) =>
                if let GuiAppPage::AddConfig(page) = &mut self.page
                    && let Some(email) = page.update(msg)
                {
                    page.loading(true);
                    let providers = Arc::clone(&self.providers);
                    let config = Arc::clone(&self.config);
                    return Task::perform(
                        Self::auth(email, providers, config),
                        |res| match res {
                            Ok(()) => GuiAppMessage::ProviderAdded,
                            Err(str) => GuiAppMessage::AddConfig(
                                AddConfigMessage::Error(str),
                            ),
                        },
                    );
                },
            GuiAppMessage::Main(()) => (),
            GuiAppMessage::ProviderAdded =>
                self.page = GuiAppPage::Main(MainPage),
            GuiAppMessage::Authenticate => {
                let config = Arc::clone(&self.config);
                let providers = Arc::clone(&self.providers);
                return Task::perform(
                    Self::auth_config(config, providers),
                    |res| match res {
                        Ok(()) => GuiAppMessage::ProviderAdded,
                        Err(str) => GuiAppMessage::AddConfig(
                            AddConfigMessage::Error(str),
                        ),
                    },
                );
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, GuiAppMessage> {
        let view = match &self.page {
            GuiAppPage::AddConfig(page) =>
                page.view().map(GuiAppMessage::AddConfig),
            GuiAppPage::Main(main) => main.view().map(GuiAppMessage::Main),
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
