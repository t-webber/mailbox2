extern crate alloc;
use alloc::sync::Arc;

use iced::Length;
use iced::widget::text::Wrapping;
use iced::widget::{Column, column, container};
use mailbox_email::EmailHeader;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::{btn, scroll, txt};
use crate::ui::style::grey;

/// Shared header.
pub type Header = ArMx<EmailHeader>;
/// Shared list of headers.
pub type Headers = ArMx<Vec<Header>>;

/// Page to display the list of headers.
#[derive(Default)]
pub struct HeadersPage {
    /// Currently opened header.
    current: Option<u32>,
    /// List of headers.
    headers: Headers,
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
}

impl Page for HeadersPage {
    type Message = HeadersMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        scroll(
            Column::with_children(lock!(self.headers).iter().map(|header| {
                let lock = lock!(header);
                debug_assert_eq!(lock.date().len(), 14, "invalid format");
                btn(
                    column!(
                        container(txt(lock.from()).wrapping(Wrapping::None))
                            .clip(true)
                            .height(Length::Shrink)
                            .width(Length::Fixed(200.)),
                        container(
                            txt(lock.subject().to_owned())
                                .wrapping(Wrapping::None)
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
