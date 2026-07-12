extern crate alloc;
use alloc::sync::Arc;

use iced::widget::pick_list;
use iced::{Renderer, Theme};
use mailbox_shared::{ArMx, lock};

use crate::Page;

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
        pick_list::<_, _, _, _, Theme, Renderer>(
            lock!(self.list()).iter().map(Arc::clone).collect::<Vec<_>>(),
            Some(Arc::clone(&self.current)),
            |val: Arc<str>| val,
        )
        .into()
    }
}

/// Message from the mailbox selector.
pub type SelectBoxMsg = Arc<str>;
