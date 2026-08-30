/// Page to add a provider configuration.
mod add_config;
/// Page provider page after authentication.
mod main;

extern crate alloc;

use alloc::sync::Arc;

use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::widget::container::Style;
use iced::widget::operation::{focus_next, focus_previous};
use iced::{Element, Length, Subscription, Task, keyboard};

pub use crate::pages::add_config::AddConfigMessage;
use crate::pages::add_config::AddConfigPage;
pub use crate::pages::main::MainPage;
use crate::pages::main::{MainMessage, SelectProviderMsg};
use crate::ui::component::txt;
use crate::{GuiApp, Page, Provider};

/// Application messages.
#[derive(Clone, Debug)]
pub enum GuiAppMessage {
    /// Messages for the add config page.
    AddConfig(AddConfigMessage),
    /// Authenticates the providers loaded from the configuration.
    Authenticate,
    /// Display an error.
    Error(&'static str),
    /// Tab.
    FocusNext,
    /// Shift tab.
    FocusPrevious,
    /// Message for the main page.
    Main(MainMessage),
    /// Nothing to be done.
    None,
    /// Message for when a provider is added.
    ProviderAdded(Provider, Option<&'static str>),
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

    fn subscription(&self) -> Subscription<Self::Message> {
        keyboard::listen().map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event
                && key == keyboard::Key::Named(Named::Tab)
            {
                if modifiers == Modifiers::SHIFT {
                    return GuiAppMessage::FocusPrevious;
                } else if modifiers == Modifiers::NONE {
                    return GuiAppMessage::FocusNext;
                }
            }
            GuiAppMessage::None
        })
    }

    fn update(&mut self, data: Self::Message) -> Self::Task {
        if let GuiAppPage::AddConfig(page) = &mut self.page {
            page.loading(false);
        }
        match data {
            GuiAppMessage::Main(MainMessage::LoadHeaders(headers)) =>
                if let GuiAppPage::Main(main) = &mut self.page {
                    return Task::done(GuiAppMessage::Main(
                        main.add_headers(&headers),
                    ));
                },
            GuiAppMessage::FocusNext => return focus_next(),
            GuiAppMessage::FocusPrevious => return focus_previous(),
            GuiAppMessage::Main(MainMessage::SelectMailbox(mailbox)) =>
                if let GuiAppPage::Main(main) = &mut self.page {
                    return main
                        .select_box_and_fetch_headers(mailbox)
                        .map(GuiAppMessage::Main);
                },
            GuiAppMessage::Main(MainMessage::Error(Some(error)))
            | GuiAppMessage::Error(error) => self.error(error),
            GuiAppMessage::AddConfig(AddConfigMessage::Cancel(current)) => {
                self.page = GuiAppPage::Main(MainPage::new(
                    current,
                    Arc::clone(&self.providers),
                ));
                if let GuiAppPage::Main(main) = &mut self.page {
                    return main.boot().map(GuiAppMessage::Main);
                }
            }
            GuiAppMessage::AddConfig(msg) => return self.add_config(msg),
            GuiAppMessage::Main(
                MainMessage::Error(None) | MainMessage::Body(()),
            )
            | GuiAppMessage::None => (),
            GuiAppMessage::Main(MainMessage::Headers(header)) =>
                if let GuiAppPage::Main(main) = &mut self.page {
                    return main.open_header(header).map(|err| {
                        err.map_or_else(
                            || GuiAppMessage::None,
                            GuiAppMessage::Error,
                        )
                    });
                },
            GuiAppMessage::Main(MainMessage::Loaded(error)) => {
                if let Some(str) = error {
                    self.error(str);
                }
                if let GuiAppPage::Main(main) = &mut self.page {
                    main.loading(false);
                }
            }
            GuiAppMessage::Main(MainMessage::SelectProvider(
                SelectProviderMsg::AddProvider(alias),
            )) => self.page = GuiAppPage::AddConfig(AddConfigPage::old(alias)),
            GuiAppMessage::Main(MainMessage::SelectProvider(
                SelectProviderMsg::SelectProvider(provider),
            )) =>
                if let GuiAppPage::Main(main) = &mut self.page {
                    main.update(provider);
                },
            GuiAppMessage::ProviderAdded(current, err) =>
                return self.add_provider(current, err),
            GuiAppMessage::Authenticate => return self.auth_and_store(),
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
