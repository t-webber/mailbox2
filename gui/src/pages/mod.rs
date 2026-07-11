/// Page to add a provider configuration.
mod add_config;
/// Page provider page after authentication.
mod main;

extern crate alloc;

use alloc::sync::Arc;
use std::sync::Mutex;

use iced::widget::container::Style;
use iced::widget::container;
use iced::{Element, Length, Task};
use mailbox_email::EmailProvider;

use crate::pages::add_config::{AddConfigMessage, AddConfigPage};
use crate::pages::main::{MainMessage, MainPage};
use crate::ui::component::txt;
use crate::{GuiApp, Page};

/// Application messages.
#[derive(Clone, Debug)]
pub enum GuiAppMessage {
    /// Messages for the add config page.
    AddConfig(AddConfigMessage),
    /// Authenticates the providers loaded from the configuration.
    Authenticate,
    /// Display an error.
    Error(&'static str),
    /// Message for the main page.
    Main(MainMessage),
    /// Nothing to be done.
    None,
    /// Message for when a provider is added.
    ProviderAdded(Arc<Mutex<EmailProvider>>, Option<&'static str>),
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
    type Task = Task<Self::Message>;
    type Update = Self::Message;

    fn update(&mut self, data: Self::Message) -> Self::Task {
        if let GuiAppPage::AddConfig(page) = &mut self.page {
            page.loading(false);
        }
        match data {
            GuiAppMessage::None => (),
            GuiAppMessage::Error(error) => self.error(error),
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
                            Ok(provider) =>
                                GuiAppMessage::ProviderAdded(provider, None),
                            Err(str) => GuiAppMessage::AddConfig(
                                AddConfigMessage::Error(str),
                            ),
                        },
                    );
                },
            GuiAppMessage::Main(MainMessage::AddProvider) =>
                self.page = GuiAppPage::AddConfig(AddConfigPage::default()),
            GuiAppMessage::Main(MainMessage::SelectProvider(provider)) =>
                if let GuiAppPage::Main(main) = &mut self.page {
                    main.update(provider);
                },
            GuiAppMessage::ProviderAdded(current, err) => {
                self.page = GuiAppPage::Main(MainPage::new(
                    current,
                    Arc::clone(&self.providers),
                ));
                if let Some(str) = err {
                    self.error(str);
                }
            }
            GuiAppMessage::Authenticate => {
                let config = Arc::clone(&self.config);
                let providers = Arc::clone(&self.providers);
                return Task::perform(
                    Self::auth_config(config, providers),
                    |res| match res {
                        (Some(first), err) =>
                            GuiAppMessage::ProviderAdded(first, err),
                        (None, Some(err)) => GuiAppMessage::Error(err),
                        (None, None) => GuiAppMessage::None,
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
            GuiAppPage::Authenticate => txt("authenticating...").into(),
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
