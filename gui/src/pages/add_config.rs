extern crate alloc;
use alloc::sync::Arc;

use iced::widget::{button, column, container, text, text_input};
use iced::{Alignment, Color, Element, Length};
use mailbox_shared::EmailConfig;

use crate::Page;

/// Page to enter an email provider configuration.
///
/// Refer to [`EmailConfig`] for more information
/// about each field.
#[derive(Default)]
#[allow(
    clippy::allow_attributes,
    clippy::missing_docs_in_private_items,
    reason = "dup doc"
)]
pub struct AddConfigPage {
    domain: Arc<str>,
    error: &'static str,
    loading: bool,
    password: Arc<str>,
    port: u16,
    user: Arc<str>,
}

impl AddConfigPage {
    /// Displays an error to the user.
    pub const fn error(&mut self, msg: &'static str) {
        self.error = msg;
    }

    /// Marks the UI as loading.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Makes an [`EmailConfig`] from the form data.
    fn to_cfg(&self) -> EmailConfig {
        EmailConfig::new(
            Arc::clone(&self.user),
            Arc::clone(&self.password),
            Arc::clone(&self.domain),
            self.port,
        )
    }
}

impl Page for AddConfigPage {
    type Message = Message;
    type Update = Option<EmailConfig>;

    fn update(&mut self, message: Self::Message) -> Self::Update {
        if self.loading {
            return None;
        }
        match message {
            Message::Domain(str) => self.domain = str,
            Message::User(usr) => self.user = usr,
            Message::Password(psk) => self.password = psk,
            Message::Port(port) =>
                if port.is_empty() {
                    self.port = 0;
                } else if let Ok(nb) = port.parse() {
                    self.port = nb;
                },
            Message::Submit =>
                if self.user.is_empty() {
                    self.error("Missing user");
                } else if self.password.is_empty() {
                    self.error("Missing password");
                } else if self.domain.is_empty() {
                    self.error("Missing domain");
                } else if self.port == 0 {
                    self.error("Missing port");
                } else {
                    return Some(self.to_cfg());
                },
        }
        None
    }

    fn view(&self) -> Element<'_, Message> {
        let cols = column!(
            text("New email provider").color(Color::WHITE),
            text_input("User", &self.user)
                .on_input(|x| Message::User(x.into())),
            text_input("Password", &self.password)
                .on_input(|x| Message::Password(x.into())),
            text_input("Domain", &self.domain)
                .on_input(|x| Message::Domain(x.into())),
            text_input(
                "Port",
                &if self.port == 0 {
                    String::new()
                } else {
                    self.port.to_string()
                }
            )
            .on_input(|x| Message::Port(x.into())),
            button("Submit").on_press(Message::Submit)
        );
        container(
            if self.loading {
                cols.push(
                    text("Establishing connection...")
                        .color(Color::from_rgb8(0xe5, 0xc0, 0x7b)),
                )
            } else if self.error.is_empty() {
                cols
            } else {
                cols.push(
                    text(self.error).color(Color::from_rgb8(0xe0, 0x6c, 0x75)),
                )
            }
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        )
        .into()
    }
}

/// Message to update one field of [`AddConfigPage`].
///
/// Refer to [`EmailConfig`] for more information
/// about each field.
#[allow(
    clippy::allow_attributes,
    clippy::missing_docs_in_private_items,
    reason = "dup doc"
)]
#[derive(Clone, Debug)]
pub enum Message {
    Domain(Arc<str>),
    Password(Arc<str>),
    Port(Arc<str>),
    Submit,
    User(Arc<str>),
}
