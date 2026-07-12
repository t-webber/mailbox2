/// Displays the body of an email.
mod body;
/// Lists the headers of the current mailbox.
mod headers;
/// Left bar to select the active provider.
mod select_provider;

use iced::widget::{container, row};
use iced::{Alignment, Length, Task};
use mailbox_email::{EmailBody, FetchHeadersError};
use mailbox_shared::{ArMx, lock};

use crate::pages::main::body::{BodyMsg, BodyPage};
pub use crate::pages::main::headers::HeadersMsg;
use crate::pages::main::headers::{Headers, HeadersPage};
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
    /// Left bar to select the active provider.
    provider_selector: SelectProviderPage,
}

impl MainPage {
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

    /// Sets the loading status.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Creates a new page.
    pub fn new(current: Provider, list: Providers) -> Self {
        Self {
            body: BodyPage::default(),
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

    /// Populate the headers and return a task.
    pub fn schedule_headers_fetching(&self) -> Task<Option<&'static str>> {
        let headers = self.headers.list();
        let provider = self.provider_selector.current();
        Task::perform(Self::fetch_headers(headers, provider), |res| res)
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
                container(self.headers.view().map(MainMessage::Headers))
                    .width(250.),
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
    /// Message from the header list.
    Headers(HeadersMsg),
    /// The headers finished loading.
    Loaded(Option<&'static str>),
    /// Message from the provider selector.
    SelectProvider(SelectProviderMsg),
}
