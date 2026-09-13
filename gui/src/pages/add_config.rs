use alloc::sync::Arc;

use iced::border::rounded;
use iced::widget::{Column, button, container, row};
use iced::{Alignment, Element, Length, Pixels};
use mailbox_shared::{EmailConfig, ErrStr, display_err, errmsg};

use crate::ui::component::{btn, input, txt};
use crate::ui::style::{
    BTN_COLOUR, FOCUSED_COLOUR, RADIUS, RED, TXT_FONT, YELLOW, grey
};
use crate::{Page, Provider};

/// Which provider type is being configured.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderType {
    /// Email provider.
    #[default]
    Email,
    /// `WhatsApp` provider.
    WhatsApp,
}

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
    error: ErrStr,
    loading: bool,
    password: Arc<str>,
    port: u16,
    previous: Option<Provider>,
    provider_type: ProviderType,
    show_password: bool,
    user: Arc<str>,
    wa_needs_pairing: bool,
}

impl AddConfigPage {
    /// Displays an error message.
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::missing_const_for_fn,
            reason = "can't be in debug+inferred by compiler"
        )
    )]
    pub fn error(&mut self, error: ErrStr) {
        self.error = error;
    }

    /// Marks the UI as loading.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Creates a new configuration adding page with no previous provider.
    pub fn new(wa_needs_pairing: bool) -> Self {
        Self { wa_needs_pairing, ..Self::default() }
    }

    /// Creates a new configuration adding page with a fallback on this char if
    /// cancelled.
    pub fn old(previous: Provider, wa_needs_pairing: bool) -> Self {
        Self { previous: Some(previous), wa_needs_pairing, ..Self::default() }
    }

    /// Returns the email/`WhatsApp` toggle.
    fn provider_chooser(&self) -> Element<'_, AddConfigMessage> {
        let is_email = self.provider_type == ProviderType::Email;
        let email_btn = btn(
            txt("Email"),
            AddConfigMessage::ProviderType(ProviderType::Email),
            is_email,
            true,
            BTN_COLOUR,
            FOCUSED_COLOUR,
            FOCUSED_COLOUR,
        );
        let whatsapp_btn = if self.wa_needs_pairing {
            btn(
                txt("WhatsApp"),
                AddConfigMessage::ProviderType(ProviderType::WhatsApp),
                !is_email,
                true,
                BTN_COLOUR,
                FOCUSED_COLOUR,
                FOCUSED_COLOUR,
            )
        } else {
            button(txt("WhatsApp")).style(|_, _| button::Style {
                background: Some(grey(30).into()),
                text_color: grey(80),
                border: rounded(RADIUS),
                ..Default::default()
            })
        };
        row![email_btn, whatsapp_btn].spacing(4.).into()
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
    type Message = AddConfigMessage;
    type Task = Option<EmailConfig>;
    type Update = Self::Message;

    fn update(&mut self, data: AddConfigMessage) -> Self::Task {
        if self.loading {
            return None;
        }
        match data {
            AddConfigMessage::ShowPassword =>
                self.show_password = !self.show_password,
            AddConfigMessage::Alias(ch) => {
                if self.alias.is_some() && ch.is_some() {
                    self.error = errmsg!(
                        "Alias can't contain more than 1 character",
                        ""
                    );
                }
                self.alias = ch;
            }
            AddConfigMessage::Domain(str) => self.domain = str,
            AddConfigMessage::User(usr) => self.user = usr,
            AddConfigMessage::Password(psk) => self.password = psk,
            AddConfigMessage::Port(port) =>
                if port.is_empty() {
                    self.port = 0;
                } else if let Ok(nb) = port.parse() {
                    self.port = nb;
                } else {
                    self.error = errmsg!(
                        "Port must be a valid unsigned 16-bits integer",
                        ""
                    );
                },
            AddConfigMessage::ProviderType(pt) => {
                self.provider_type = pt;
                self.error = errmsg!("", "");
            }
            AddConfigMessage::Submit => {
                self.error = errmsg!("", "");
                if self.alias.is_none() {
                    self.error = errmsg!("Missing alias", "");
                } else if self.user.is_empty() {
                    self.error = errmsg!("Missing user", "");
                } else if self.password.is_empty() {
                    self.error = errmsg!("Missing password", "");
                } else if self.domain.is_empty() {
                    self.error = errmsg!("Missing domain", "");
                } else if self.port == 0 {
                    self.error = errmsg!("Missing port", "");
                } else {
                    return Some(self.to_cfg());
                }
            }
            AddConfigMessage::Error(error) => self.error = error,
            AddConfigMessage::Cancel(_) => (),
        }
        None
    }

    fn view(&self) -> Element<'_, AddConfigMessage> {
        let submit = btn(
            txt("Submit"),
            AddConfigMessage::Submit,
            false,
            true,
            BTN_COLOUR,
            FOCUSED_COLOUR,
            FOCUSED_COLOUR,
        );
        let elements: [Element<'_, AddConfigMessage>; 9] = [
            txt("New email provider").size(TXT_FONT + 2).into(),
            self.provider_chooser(),
            input(
                "Alias for displaying it in this app",
                &self.alias.map(|ch| ch.to_string()).unwrap_or_default(),
                |x: String| AddConfigMessage::Alias(x.chars().last()),
                false,
            )
            .into(),
            input("User (email)", &self.user, AddConfigMessage::User, false)
                .into(),
            row![
                input(
                    "Password",
                    &self.password,
                    AddConfigMessage::Password,
                    !self.show_password,
                ),
                btn(
                    txt("\u{f0208}"),
                    AddConfigMessage::ShowPassword,
                    false,
                    true,
                    BTN_COLOUR,
                    FOCUSED_COLOUR,
                    FOCUSED_COLOUR
                )
            ]
            .spacing(Pixels(2.))
            .into(),
            input(
                "Domain (e.g. imap.gmail.com)",
                &self.domain,
                AddConfigMessage::Domain,
                false,
            )
            .into(),
            input(
                "Port (e.g. 993)",
                &if self.port == 0 {
                    String::new()
                } else {
                    self.port.to_string()
                },
                AddConfigMessage::Port,
                false,
            )
            .into(),
            if let Some(prev) = &self.previous {
                row!(
                    submit,
                    btn(
                        txt("Cancel"),
                        AddConfigMessage::Cancel(prev.clone()),
                        false,
                        true,
                        grey(50),
                        grey(100),
                        grey(100),
                    ),
                )
                .spacing(4.)
                .into()
            } else {
                submit.into()
            },
            if self.loading {
                txt("Establishing connection...").color(YELLOW)
            } else if self.error.is_empty() {
                txt("")
            } else {
                txt(display_err!(self.error)).color(RED)
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
        .width(Length::Fixed(300.))
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
pub enum AddConfigMessage {
    Alias(Option<char>),
    Cancel(Provider),
    Domain(Arc<str>),
    Error(ErrStr),
    Password(Arc<str>),
    Port(Arc<str>),
    ProviderType(ProviderType),
    ShowPassword,
    Submit,
    User(Arc<str>),
}
