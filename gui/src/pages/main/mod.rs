/// Lists the headers of the current mailbox.
mod headers;
/// Left bar to select the active provider.
mod select_provider;
use iced::widget::{Space, container, row};
use iced::{Alignment, Length, Task};
use mailbox_email::FetchHeadersError;
use mailbox_shared::lock;

pub use crate::pages::main::headers::HeadersMsg;
use crate::pages::main::headers::{Headers, HeadersPage};
pub use crate::pages::main::select_provider::SelectProviderMsg;
use crate::pages::main::select_provider::SelectProviderPage;
use crate::ui::component::txt;
use crate::{Page, Provider, Providers};

/// Main page for one provider.
pub struct MainPage {
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

    /// Sets the loading status.
    pub const fn loading(&mut self, loading: bool) {
        self.loading = loading;
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
                let mut guard = lock!(headers);
                *guard = new_headers;
                drop(guard);
                errors.first().map(FetchHeadersError::display)
            }
            Err(error) => Some(error.display()),
        }
    }

    /// Creates a new page.
    pub fn new(current: Provider, list: Providers) -> Self {
        Self {
            provider_selector: SelectProviderPage::new(current, list),
            headers: HeadersPage::default(),
            error: None,
            loading: true,
        }
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
                    .width(300.),
                Space::new().width(Length::Fill)
            )
        }
        .into()
    }
}

/// Message for the main provider panel.
#[derive(Clone, Debug)]
pub enum MainMessage {
    /// The headers finished loading.
    Loaded(Option<&'static str>),
    /// Message from the header list.
    Headers(HeadersMsg),
    /// Message from the provider selector.
    SelectProvider(SelectProviderMsg),
}
