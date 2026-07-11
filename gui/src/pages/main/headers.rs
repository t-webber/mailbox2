use iced::widget::Column;
use mailbox_email::EmailHeader;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::txt;

/// Shared header.
type Header = ArMx<EmailHeader>;
/// Shared list of headers.
type Headers = ArMx<Vec<Header>>;

/// Page to display the list of headers.
#[derive(Default)]
pub struct HeadersPage {
    /// List of headers.
    headers: Headers,
}

impl Page for HeadersPage {
    type Message = HeadersMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::with_children(
            lock!(self.headers)
                .iter()
                .map(|header| txt(lock!(header).subject().to_owned()).into()),
        )
        .into()
    }
}

/// Message returned by the headers list.
pub type HeadersMsg = ();
