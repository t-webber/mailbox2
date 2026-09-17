use alloc::sync::Arc;

use iced::border::rounded;
use iced::widget::{button, container, row};
use iced::{Alignment, Element, Length, Pixels};
use mailbox_shared::{EmailConfig, ErrStr, display_err, errmsg};
use mailbox_whatsapp::Whatsapp;

use crate::ui::component::{btn, input, padded_column, txt};
use crate::ui::style::{
    BTN_COLOUR, FOCUSED_COLOUR, RADIUS, RED, TXT_FONT, YELLOW, grey
};
use crate::{Page, Provider};

/// New configuration just created from the form.
#[expect(dead_code, reason = "todo")]
pub enum NewConfig {
    /// Email configuration.
    Email(EmailConfig),
    /// `WhatsApp` configuration.
    Whatsapp(char),
}

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
    wa_login_code: Option<Arc<str>>,
    wa_needs_pairing: bool,
}

impl AddConfigPage {
    /// Input to fill to specify alias of provider.
    fn alias_input(&self) -> Element<'_, AddConfigMessage> {
        input(
            "Alias for displaying it in this app",
            &self.alias.map(|ch| ch.to_string()).unwrap_or_default(),
            |x: String| AddConfigMessage::Alias(x.chars().last()),
            false,
        )
        .into()
    }

    /// Form to fill to create an email provider.
    fn email_form(&self) -> [Element<'_, AddConfigMessage>; 5] {
        [
            self.alias_input(),
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
        ]
    }

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

    /// Authenticate by pairing a whatsapp device.
    fn try_auth_whatsapp(&mut self) -> Option<NewConfig> {
        if let Err(msg) = Whatsapp::validate_phone(&self.user) {
            self.error = errmsg!(msg);
            None
        } else if let Some(alias) = self.alias {
            Some(NewConfig::Whatsapp(alias))
        } else {
            self.error = errmsg!("Missing alias");
            None
        }
    }

    /// Makes an [`EmailConfig`] from the form data.
    fn try_make_email_cfg(&mut self) -> Option<NewConfig> {
        let Some(alias) = self.alias else {
            self.error = errmsg!("Missing alias");
            return None;
        };
        if self.user.is_empty() {
            self.error = errmsg!("Missing user");
        } else if self.password.is_empty() {
            self.error = errmsg!("Missing password");
        } else if self.domain.is_empty() {
            self.error = errmsg!("Missing domain");
        } else if self.port == 0 {
            self.error = errmsg!("Missing port");
        } else {
            return Some(NewConfig::Email(EmailConfig::new(
                alias,
                Arc::clone(&self.user),
                Arc::clone(&self.password),
                Arc::clone(&self.domain),
                self.port,
            )));
        }
        None
    }

    /// Form to fill to create a whatsapp provider.
    fn wa_form(&self) -> [Element<'_, AddConfigMessage>; 2] {
        [
            self.alias_input(),
            input(
                "Phone number (w/ country code, w/o leading 0)",
                &self.user,
                AddConfigMessage::User,
                false,
            )
            .into(),
        ]
    }
}

impl Page for AddConfigPage {
    type Message = AddConfigMessage;
    type Task = Option<NewConfig>;
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
                    self.error =
                        errmsg!("Alias can't contain more than 1 character");
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
                        "Port must be a valid unsigned 16-bits integer"
                    );
                },
            AddConfigMessage::ProviderType(pt) => {
                self.provider_type = pt;
                self.error = ErrStr::default();
            }
            AddConfigMessage::Submit => {
                self.error = ErrStr::default();
                match self.provider_type {
                    ProviderType::Email => return self.try_make_email_cfg(),
                    ProviderType::WhatsApp => return self.try_auth_whatsapp(),
                }
            }
            AddConfigMessage::Error(error) => self.error = error,
            AddConfigMessage::Cancel(_) => (),
            AddConfigMessage::WaLoginCode(code) =>
                self.wa_login_code = Some(code),
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
        let elements: [Element<'_, AddConfigMessage>; 5] = [
            txt("New email provider").size(TXT_FONT + 2).into(),
            self.provider_chooser(),
            match self.provider_type {
                ProviderType::Email => padded_column(self.email_form()).into(),
                ProviderType::WhatsApp => padded_column(self.wa_form()).into(),
            },
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
        container(padded_column(elements).align_x(Alignment::Center))
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
    WaLoginCode(Arc<str>),
}
