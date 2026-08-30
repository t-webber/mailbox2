
use alloc::sync::Arc;

use iced::widget::text::{Rich, Span};
use iced::widget::{container, rich_text, span};
use iced::{Color, Element, Font, Length};
use linkify::{LinkFinder, LinkKind};
use mailbox_email::EmailBody;
use mailbox_shared::{ArMx, lock};

use crate::Page;
use crate::ui::component::{scroll, txt};
use crate::ui::style::{LINK_COLOUR, TXT_COLOUR, TXT_FONT, YELLOW};

/// Page to display the list of headers.
#[derive(Default)]
pub enum BodyPage {
    /// No email selected.
    #[default]
    None,
    /// Email to display.
    ///
    /// If None, the body is loading.
    Some(ArMx<Option<EmailBody>>),
}

impl BodyPage {
    /// Returns the list of headers.
    pub fn loading(&mut self) -> ArMx<Option<EmailBody>> {
        let body = Arc::default();
        *self = Self::Some(Arc::clone(&body));
        body
    }

    /// Displays the body of an email.
    fn view_body(body: &EmailBody) -> Rich<'static, String, BodyMsg> {
        let mut spans: Vec<Span<'static, String>> = vec![];
        let plain = body.plain().trim();
        let mut prev_end = 0;
        for link in LinkFinder::new().links(plain) {
            spans.push(
                span(filter_whitespace(
                    plain.get(prev_end..link.start()).unwrap_or_default(),
                ))
                .color(TXT_COLOUR)
                .font(Font::MONOSPACE),
            );
            prev_end = link.end();
            spans.push(match link.kind() {
                LinkKind::Url => span(
                    link.as_str()
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_start_matches("www.")
                        .split('/')
                        .next()
                        .unwrap_or_default()
                        .split('?')
                        .next()
                        .unwrap_or_default()
                        .to_owned(),
                )
                .color(LINK_COLOUR)
                .link(link.as_str()),
                LinkKind::Email => span(link.as_str().to_owned())
                    .color(LINK_COLOUR)
                    .link(format!("mailto:{}", link.as_str())),
                _ => span(link.as_str().to_owned()),
            });
        }
        spans.push(
            span(filter_whitespace(plain.get(prev_end..).unwrap_or_default()))
                .color(TXT_COLOUR),
        );
        rich_text(spans)
            .size(TXT_FONT)
            .font(Font::MONOSPACE)
            .on_link_click(|url| drop(open::that(url)))
    }
}

impl Page for BodyPage {
    type Message = BodyMsg;
    type Task = ();
    type Update = ();

    fn update(&mut self, (): Self::Update) -> Self::Task {}

    fn view(&self) -> Element<'_, Self::Message> {
        container(match self {
            Self::None => txt("Select an email to display it"),
            Self::Some(maybe_body) => match lock!(maybe_body).as_ref() {
                Some(body) =>
                    return scroll(Self::view_body(body), Color::BLACK).into(),
                None => txt("Loading").color(YELLOW),
            },
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

/// Message sent after interactive with the body pane.
pub type BodyMsg = ();

/// Removes successive whitespaces and newlines.
fn filter_whitespace(raw: &str) -> String {
    let start = raw.chars().next().is_some_and(is_non_newline_whitespace);
    let end = raw.chars().next_back().is_some_and(is_non_newline_whitespace);
    let lines = raw
        .split('\n')
        .fold((vec![], false), |(mut lines, previous_empty), line| {
            let trimmed =
                line.split_whitespace().collect::<Vec<&str>>().join(" ");
            let empty = trimmed.is_empty();
            if !trimmed.is_empty() || !previous_empty {
                lines.push(trimmed);
            }
            (lines, empty)
        })
        .0
        .join("\n");
    format!(
        "{}{lines}{}",
        if start { " " } else { "" },
        if end { " " } else { "" }
    )
}

/// Returns true if the char is a whitespace but not a newline.
const fn is_non_newline_whitespace(ch: char) -> bool {
    ch.is_whitespace() && ch != '\n'
}
