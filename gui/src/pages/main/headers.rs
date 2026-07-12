extern crate alloc;
use alloc::sync::Arc;
use std::collections::HashSet;

use iced::Length;
use iced::widget::text::Wrapping;
use iced::widget::{Column, column, container};
use mailbox_email::EmailHeader;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::{btn, scroll, txt};
use crate::ui::style::{TXT_COLOUR, UNSEEN_COLOUR, YELLOW, grey};

/// Shared header.
pub type Header = ArMx<EmailHeader>;
/// Shared list of headers.
///
/// None means that it is loading, Some(vec![]) means the mailbox is empty.
pub type Headers = ArMx<Option<Vec<Header>>>;

/// Page to display the list of headers.
#[derive(Default)]
pub struct HeadersPage {
    /// Currently opened header.
    current: Option<u32>,
    /// List of headers.
    headers: Headers,
    /// List of unseen emails.
    unseen: ArMx<HashSet<u32>>,
}

impl HeadersPage {
    /// Returns the list of headers.
    pub fn list(&self) -> Headers {
        Arc::clone(&self.headers)
    }

    /// Sets the currently opened header.
    pub const fn set_current(&mut self, uid: u32) {
        self.current = Some(uid);
    }

    /// Returns the set of unseen emails.
    pub fn unseen(&self) -> ArMx<HashSet<u32>> {
        Arc::clone(&self.unseen)
    }
}

impl Page for HeadersPage {
    type Message = HeadersMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let unseen = lock!(self.unseen);
        let Some(headers) = &*lock!(self.headers) else {
            return container(txt("Loading headers...").color(YELLOW))
                .center(Length::Fill)
                .width(Length::Fixed(200.))
                .into();
        };
        if headers.is_empty() {
            return container(txt("No emails in this folder."))
                .center(Length::Fill)
                .width(Length::Fixed(200.))
                .into();
        }
        scroll(
            Column::with_children(headers.iter().map(|header| {
                let lock = lock!(header);
                debug_assert_eq!(lock.date().len(), 14, "invalid format");
                let colour = if unseen.contains(&lock.uid) {
                    UNSEEN_COLOUR
                } else {
                    TXT_COLOUR
                };
                btn(
                    column!(
                        container(
                            txt(lock.from())
                                .wrapping(Wrapping::None)
                                .color(colour)
                        )
                        .clip(true)
                        .height(Length::Shrink)
                        .width(Length::Fixed(200.)),
                        container(
                            txt(lock.subject().to_owned())
                                .wrapping(Wrapping::None)
                                .color(colour)
                        )
                        .width(Length::Fixed(200.))
                        .clip(true)
                        .height(Length::Shrink)
                    ),
                    lock.uid,
                    Some(lock.uid) == self.current,
                    false,
                    grey(30),
                    grey(60),
                )
                .into()
            })),
            grey(30),
        )
        .into()
    }
}

/// Message returned by the headers list.
pub type HeadersMsg = u32;
