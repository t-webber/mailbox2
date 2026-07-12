/// Displays the body of an email.
mod body;
/// Lists the headers of the current mailbox.
mod headers;
/// Dropdown to select a folder.
mod select_box;
/// Left bar to select the active provider.
mod select_provider;

extern crate alloc;
use alloc::sync::Arc;
use std::collections::HashSet;

use iced::widget::{column, container, row};
use iced::{Alignment, Length, Task};
use mailbox_email::{EmailBody, FetchHeadersError, ListBoxError};
use mailbox_shared::{ArMx, lock};

pub use crate::pages::main::body::BodyMsg;
use crate::pages::main::body::BodyPage;
pub use crate::pages::main::headers::HeadersMsg;
use crate::pages::main::headers::{Headers, HeadersPage};
use crate::pages::main::select_box::{Mailboxes, SelectBoxPage};
pub use crate::pages::main::select_provider::SelectProviderMsg;
use crate::pages::main::select_provider::SelectProviderPage;
use crate::ui::component::txt;
use crate::{Page, Provider, Providers};

/// Main page for one provider.
pub struct MainPage {
    /// Body to display.
    body: BodyPage,
    /// Error to display.
    error: Option<&'static str>,
    /// List of headers.
    headers: HeadersPage,
    /// Whether the app is ready to show data or not.
    loading: bool,
    /// Dropdown to select the active mailbox.
    mailbox_selector: SelectBoxPage,
    /// Left bar to select the active provider.
    provider_selector: SelectProviderPage,
}

impl MainPage {
    /// Populates the main page data with asynchronous workers.
    pub fn boot(&mut self) -> Task<MainMessage> {
        let provider = self.provider_selector.current();
        let boxes = self.mailbox_selector.list();
        Task::batch([
            self.select_box_and_fetch_headers("INBOX".into()),
            Task::perform(
                Self::fetch_boxes(boxes, provider),
                MainMessage::Error,
            ),
        ])
    }

    /// Displays an error message.
    pub const fn error(&mut self, error: &'static str) {
        self.error = Some(error);
    }

    /// Fetch all the headers for the current mailbox.
    ///
    /// Returns the first error if any.
    async fn fetch_body(
        uid: u32,
        body: ArMx<Option<EmailBody>>,
        provider: Provider,
    ) -> Option<&'static str> {
        match provider.get_body(uid).await {
            Ok(new_body) => {
                *lock!(body) = Some(new_body);
                None
            }
            Err(error) => Some(error.display()),
        }
    }

    /// Fetch the list of mailboxes.
    ///
    /// Returns the first error if any.
    async fn fetch_boxes(
        boxes: Mailboxes,
        provider: Provider,
    ) -> Option<&'static str> {
        match provider.get_mailboxes().await {
            Ok((list, errors)) => {
                *lock!(boxes) = list;
                errors.first().map(ListBoxError::display)
            }
            Err(error) => Some(error.display()),
        }
    }

    /// Fetch all the headers for the current mailbox.
    ///
    /// Returns the first error if any.
    async fn fetch_headers(
        headers: Headers,
        provider: Provider,
    ) -> Option<&'static str> {
        match provider.get_headers().await {
            Ok((new_headers, errors)) => {
                *lock!(headers) = new_headers;
                errors.first().map(FetchHeadersError::display)
            }
            Err(error) => Some(error.display()),
        }
    }

    /// Fetch the list of unseen emails.
    async fn fetch_unseen(
        unseen: ArMx<HashSet<u32>>,
        provider: Provider,
    ) -> Option<&'static str> {
        match provider.get_unseen().await {
            Ok(new) => {
                *lock!(unseen) = new;
                None
            }
            Err(err) => Some(err.display()),
        }
    }

    /// Sets the loading status.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Creates a new page.
    pub fn new(current: Provider, list: Providers) -> Self {
        Self {
            body: BodyPage::default(),
            mailbox_selector: SelectBoxPage::new("INBOX".into()),
            provider_selector: SelectProviderPage::new(current, list),
            headers: HeadersPage::default(),
            error: None,
            loading: true,
        }
    }

    /// Fetches the body for that header and displays it.
    pub fn open_header(&mut self, uid: u32) -> Task<Option<&'static str>> {
        self.headers.set_current(uid);
        let body = self.body.loading();
        let provider = self.provider_selector.current();
        Task::perform(Self::fetch_body(uid, body, provider), |res| res)
    }

    /// Selects a new mailbox.
    ///
    /// This also fetches the list of headers and unseen messages once the
    /// mailbox is selected.
    async fn select_box(
        provider: Provider,
        name: Arc<str>,
    ) -> Option<&'static str> {
        provider
            .select_mailbox(&name)
            .await
            .map_or_else(|err| Some(err.display()), |()| None)
    }

    /// Selects a new mailbox.
    ///
    /// This also fetches the list of headers and unseen messages once the
    /// mailbox is selected.
    pub fn select_box_and_fetch_headers(
        &mut self,
        name: Arc<str>,
    ) -> Task<MainMessage> {
        self.mailbox_selector.update(Arc::clone(&name));
        let headers = self.headers.list();
        let unseen = self.headers.unseen();
        let provider = self.provider_selector.current();
        let provider1 = provider.clone();
        Task::perform(Self::select_box(provider1, name), MainMessage::Error)
            .then(move |_| {
                Task::batch([
                    Task::perform(
                        Self::fetch_headers(
                            Arc::clone(&headers),
                            provider.clone(),
                        ),
                        MainMessage::Loaded,
                    ),
                    Task::perform(
                        Self::fetch_unseen(
                            Arc::clone(&unseen),
                            provider.clone(),
                        ),
                        MainMessage::Error,
                    ),
                ])
            })
    }
}

impl Page for MainPage {
    type Message = MainMessage;
    type Task = ();
    type Update = Provider;

    fn update(&mut self, data: Self::Update) -> Self::Task {
        self.provider_selector.update(data);
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let providers =
            self.provider_selector.view().map(MainMessage::SelectProvider);
        if self.loading {
            row!(
                providers,
                container(
                    txt("loading...")
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                )
            )
        } else {
            row!(
                providers,
                column!(
                    container(
                        self.mailbox_selector
                            .view()
                            .map(MainMessage::SelectMailbox)
                    ),
                    self.headers.view().map(MainMessage::Headers)
                ),
                txt(" "),
                container(self.body.view().map(MainMessage::Body))
                    .width(Length::Fill),
            )
        }
        .into()
    }
}

/// Message for the main provider panel.
#[derive(Clone, Debug)]
pub enum MainMessage {
    /// Message from the email body pane.
    Body(BodyMsg),
    /// Maybe an error occurred.
    Error(Option<&'static str>),
    /// Message from the header list.
    Headers(HeadersMsg),
    /// The headers finished loading.
    Loaded(Option<&'static str>),
    /// Select a mailbox.
    SelectMailbox(Arc<str>),
    /// Message from the provider selector.
    SelectProvider(SelectProviderMsg),
}
