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
pub type Header = Arc<EmailHeader>;

/// Page to display the list of headers.
#[derive(Default)]
pub struct HeadersPage {
    /// Currently opened header.
    current: Option<u32>,
    /// List of headers.
    headers: Option<Vec<Header>>,
    /// List of unseen emails.
    unseen: ArMx<HashSet<u32>>,
}

impl HeadersPage {
    /// Empties the header list.
    pub fn empty(&mut self) {
        self.headers = None;
    }

    /// Extends the list of headers with some new ones.
    pub fn extend<I: IntoIterator<Item = Header>>(&mut self, iter: I) {
        if let Some(vec) = &mut self.headers {
            vec.extend(iter);
        } else {
            self.headers = Some(iter.into_iter().collect());
        }
    }

    /// Returns true if there is at least one header.
    pub fn has_some(&self) -> bool {
        self.headers.as_ref().is_some_and(|x| !x.is_empty())
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
        let Some(headers) = &self.headers else {
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
                debug_assert_eq!(header.date().len(), 14, "invalid format");
                let colour = if unseen.contains(&header.uid) {
                    UNSEEN_COLOUR
                } else {
                    TXT_COLOUR
                };
                btn(
                    column!(
                        container(
                            txt(header.from())
                                .wrapping(Wrapping::None)
                                .color(colour)
                        )
                        .clip(true)
                        .height(Length::Shrink)
                        .width(Length::Fixed(200.)),
                        container(
                            txt(header.subject().to_owned())
                                .wrapping(Wrapping::None)
                                .color(colour)
                        )
                        .width(Length::Fixed(200.))
                        .clip(true)
                        .height(Length::Shrink)
                    ),
                    header.uid,
                    Some(header.uid) == self.current,
                    false,
                    grey(30),
                    grey(60),
                    grey(90),
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
