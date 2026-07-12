extern crate alloc;
use alloc::sync::Arc;

use iced::widget::container;
use iced::{Element, Length};
use mailbox_email::EmailBody;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::txt;
use crate::ui::style::YELLOW;

/// Page to display the list of headers.
#[derive(Default)]
pub enum BodyPage {
    /// No email selected.
    #[default]
    None,
    /// Email to display.
    ///
    /// If None, the body is loading.
    Some(ArMx<Option<EmailBody>>),
}

impl BodyPage {
    /// Returns the list of headers.
    pub fn loading(&mut self) -> ArMx<Option<EmailBody>> {
        let body = Arc::default();
        *self = Self::Some(Arc::clone(&body));
        body
    }
}

impl Page for BodyPage {
    type Message = BodyMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> Element<'_, Self::Message> {
        container(match self {
            Self::None => txt("Select an email to display it"),
            Self::Some(maybe_body) => match lock!(maybe_body).as_ref() {
                Some(body) => return txt(body.debug()).into(),
                None => txt("Loading").color(YELLOW),
            },
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

/// Message sent after interactive with the body pane.
pub type BodyMsg = ();
