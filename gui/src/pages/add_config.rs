extern crate alloc;
use alloc::sync::Arc;

use iced::widget::{Column, container};
use iced::{Alignment, Element, Length};
use mailbox_shared::EmailConfig;

use crate::Page;
use crate::ui::component::{btn, input, txt};
use crate::ui::style::{RED, TXT_FONT, YELLOW};

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
    alias: Option<char>,
    domain: Arc<str>,
    error: &'static str,
    loading: bool,
    password: Arc<str>,
    port: u16,
    user: Arc<str>,
}

impl AddConfigPage {
    /// Marks the UI as loading.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Makes an [`EmailConfig`] from the form data.
    fn to_cfg(&self) -> EmailConfig {
        EmailConfig::new(
            self.alias.unwrap_or_default(),
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
            Message::Alias(ch) => {
                if self.alias.is_some() && ch.is_some() {
                    self.error = "Alias can't contain more than 1 character";
                }
                self.alias = ch;
            }
            Message::Domain(str) => self.domain = str,
            Message::User(usr) => self.user = usr,
            Message::Password(psk) => self.password = psk,
            Message::Port(port) =>
                if port.is_empty() {
                    self.port = 0;
                } else if let Ok(nb) = port.parse() {
                    self.port = nb;
                } else {
                    self.error =
                        "Port must be a valid unsigned 16-bits integer";
                },
            Message::Submit =>
                if self.alias.is_none() {
                    self.error = "Missing alias";
                } else if self.user.is_empty() {
                    self.error = "Missing user";
                } else if self.password.is_empty() {
                    self.error = "Missing password";
                } else if self.domain.is_empty() {
                    self.error = "Missing domain";
                } else if self.port == 0 {
                    self.error = "Missing port";
                } else {
                    return Some(self.to_cfg());
                },
            Message::Error(error) => self.error = error,
        }
        None
    }

    fn view(&self) -> Element<'_, Message> {
        let elements: [Element<'_, Message>; 8] = [
            txt("New email provider").size(TXT_FONT + 2).into(),
            input(
                "Alias for displaying it in this app",
                &self.alias.map(|ch| ch.to_string()).unwrap_or_default(),
                |x: String| Message::Alias(x.chars().last()),
            )
            .into(),
            input("User (email)", &self.user, Message::User).into(),
            input("Password", &self.password, Message::Password).into(),
            input(
                "Domain (e.g. imap.gmail.com)",
                &self.domain,
                Message::Domain,
            )
            .into(),
            input(
                "Port (e.g. 993)",
                &if self.port == 0 {
                    String::new()
                } else {
                    self.port.to_string()
                },
                Message::Port,
            )
            .into(),
            btn(txt("Submit"), Message::Submit).into(),
            if self.loading {
                txt("Establishing connection...").color(YELLOW)
            } else if self.error.is_empty() {
                txt("")
            } else {
                txt(self.error).color(RED)
            }
            .into(),
        ];
        container(
            Column::with_children(
                elements
                    .into_iter()
                    .map(|elt| container(elt).padding(2.).into()),
            )
            .align_x(Alignment::Center),
        )
        .center_x(Length::Fixed(300.))
        .center_y(Length::Fill)
        .height(Length::Fill)
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
    Alias(Option<char>),
    Domain(Arc<str>),
    Error(&'static str),
    Password(Arc<str>),
    Port(Arc<str>),
    Submit,
    User(Arc<str>),
}
