extern crate alloc;
use alloc::sync::Arc;
use core::iter::repeat_n;

use iced::Length;
use iced::widget::{Column, Space, column, row};
use mailbox_email::EmailHeader;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::{btn, txt};

/// Shared header.
pub type Header = ArMx<EmailHeader>;
/// Shared list of headers.
pub type Headers = ArMx<Vec<Header>>;

/// Page to display the list of headers.
#[derive(Default)]
pub struct HeadersPage {
    /// List of headers.
    headers: Headers,
}

impl HeadersPage {
    /// Returns the list of headers.
    pub fn list(&self) -> Headers {
        Arc::clone(&self.headers)
    }
}

impl Page for HeadersPage {
    type Message = HeadersMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::with_children(lock!(self.headers).iter().map(|header| {
            let lock = lock!(header);
            debug_assert_eq!(lock.date().len(), 14, "invalid format");
            btn(
                column!(
                    row!(
                        txt(truncate(&lock.from(), 30)),
                        Space::new().width(Length::Fill),
                        txt(lock.date())
                    )
                    .width(Length::Fill),
                    txt(truncate(lock.subject(), 30 + 14 + 2))
                )
                .width(Length::Fill),
                Arc::clone(header),
                false,
                false,
            )
            .into()
        }))
        .into()
    }
}

/// Message returned by the headers list.
pub type HeadersMsg = ArMx<EmailHeader>;

/// Truncate a string and add ellipsis.
fn truncate(raw: &str, len: usize) -> String {
    let mut chars = raw.chars();
    let mut res = String::with_capacity(len);
    let mut nb_chars = 0;
    while let Some(ch) = chars.next()
        && nb_chars < len
    {
        nb_chars = nb_chars.saturating_add(1);
        res.push(ch);
    }
    if chars.next().is_some() {
        res.push('\u{2026}');
    } else {
        res.extend(repeat_n(
            ' ',
            len.saturating_sub(nb_chars).saturating_add(1),
        ));
    }
    res
}
