extern crate alloc;
use alloc::sync::Arc;

use iced::overlay::menu;
use iced::widget::{container, pick_list};
use iced::{Length, Renderer, Shadow, Theme, border};
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::style::{
    BTN_COLOUR, FOCUSED_COLOUR, FONT, RADIUS, RED, TXT_COLOUR, TXT_FONT
};

/// Short hand for a sharable list of mailbox names.
pub type Mailboxes = ArMx<Vec<Arc<str>>>;

/// Dropdown to select a mailbox..
pub struct SelectBoxPage {
    /// Mailbox currently in use.
    current: Arc<str>,
    /// List of mailboxes.
    list: Mailboxes,
}

impl SelectBoxPage {
    /// Returns the list of mailboxes.
    pub fn list(&self) -> Mailboxes {
        Arc::clone(&self.list)
    }

    /// Creates a new [`SelectBoxPage`].
    pub fn new(current: Arc<str>) -> Self {
        Self { current, list: Arc::default() }
    }
}

impl Page for SelectBoxPage {
    type Message = SelectBoxMsg;
    type Task = ();
    type Update = Arc<str>;

    fn update(&mut self, data: Self::Update) -> Self::Task {
        self.current = data;
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(
            pick_list::<_, _, _, _, Theme, Renderer>(
                lock!(self.list()).iter().map(Arc::clone).collect::<Vec<_>>(),
                Some(Arc::clone(&self.current)),
                |val: Arc<str>| val,
            )
            .width(Length::Shrink)
            .text_size(TXT_FONT)
            .font(FONT)
            .style(|_, st| pick_list::Style {
                text_color: TXT_COLOUR,
                placeholder_color: RED,
                handle_color: TXT_COLOUR,
                background: if matches!(st, pick_list::Status::Active) {
                    BTN_COLOUR
                } else {
                    FOCUSED_COLOUR
                }
                .into(),
                border: border::rounded(RADIUS),
            })
            .menu_style(|_| menu::Style {
                background: BTN_COLOUR.into(),
                border: border::rounded(RADIUS),
                text_color: TXT_COLOUR,
                selected_text_color: TXT_COLOUR,
                selected_background: FOCUSED_COLOUR.into(),
                shadow: Shadow::default(),
            }),
        )
        .center_x(Length::Fixed(200.))
        .into()
    }
}

/// Message from the mailbox selector.
pub type SelectBoxMsg = Arc<str>;
