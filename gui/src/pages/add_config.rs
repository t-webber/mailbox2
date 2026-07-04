extern crate alloc;
use alloc::sync::Arc;

use iced::Element;
use iced::widget::{button, column, text, text_input};
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
    password: Arc<str>,
    port: u16,
    user: Arc<str>,
}

impl AddConfigPage {
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
            Message::Submit => return Some(self.to_cfg()),
        }
        None
    }

    fn view(&self) -> Element<'_, Message> {
        column!(
            text("New email provider"),
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
#[derive(Clone)]
pub enum Message {
    Domain(Arc<str>),
    Password(Arc<str>),
    Port(Arc<str>),
    Submit,
    User(Arc<str>),
}
