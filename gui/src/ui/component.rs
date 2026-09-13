use iced::border::{self, rounded};
use iced::widget::scrollable::{Direction, Rail, Scrollbar, Scroller};
use iced::widget::text::IntoFragment;
use iced::widget::{
    Button, Column, Scrollable, Text, TextInput, button, container, scrollable, text, text_input
};
use iced::{Color, Element, Font, Renderer, Theme};

use crate::ui::style::{
    BORDER_WIDTH, FOCUSED_COLOUR, FONT, RADIUS, TXT_COLOUR, TXT_FONT, grey
};

/// Text input.
pub fn input<
    'msg,
    Msg: Clone,
    MsgContent: From<String>,
    OnInput: Fn(MsgContent) -> Msg + 'msg,
>(
    placeholder: &str,
    value: &str,
    on_input: OnInput,
    secure: bool,
) -> TextInput<'msg, Msg> {
    use text_input::Status as St;
    text_input(placeholder, value)
        .secure(secure)
        .on_input(move |new| on_input(new.into()))
        .font(Font::MONOSPACE)
        .size(TXT_FONT)
        .style(|theme, st| text_input::Style {
            placeholder: grey(100),
            value: TXT_COLOUR,
            border: border::color(match st {
                St::Active => grey(50),
                St::Hovered | St::Focused { is_hovered: true } => grey(150),
                St::Disabled | St::Focused { is_hovered: false } => grey(100),
            })
            .rounded(RADIUS)
            .width(BORDER_WIDTH),
            background: grey(50).into(),
            ..text_input::default(theme, st)
        })
}

/// Button.
pub fn btn<
    'disp,
    Msg: Clone,
    Content: Into<Element<'disp, Msg, Theme, Renderer>>,
>(
    content: Content,
    msg: Msg,
    current: bool,
    round: bool,
    active_bg: Color,
    focused_bg: Color,
    current_bg: Color,
) -> Button<'disp, Msg> {
    button(content).on_press(msg).style(move |_, st| button::Style {
        background: Some(
            if current {
                current_bg
            } else if matches!(st, button::Status::Active) {
                active_bg
            } else {
                focused_bg
            }
            .into(),
        ),
        border: rounded(if round { RADIUS } else { 0. }),
        ..Default::default()
    })
}

/// Displays a simple piece of text.
pub fn txt<'txt, Content: IntoFragment<'txt>>(
    content: Content,
) -> Text<'txt, Theme, Renderer> {
    text(content).font(FONT).size(TXT_FONT).color(TXT_COLOUR)
}

/// Wraps an element in a scrollable zone.
pub fn scroll<
    'disp,
    Content: Into<Element<'disp, Msg, Theme, Renderer>>,
    Msg: 'disp,
>(
    content: Content,
    _bg: Color,
) -> Scrollable<'disp, Msg> {
    scrollable(content)
        .direction(Direction::Vertical(
            Scrollbar::new().width(0.).scroller_width(9.),
        ))
        .style(move |theme, status| scrollable::Style {
            container: container::Style::default(),
            vertical_rail: Rail {
                background: None,
                border: border::rounded(0.),
                scroller: Scroller {
                    background: FOCUSED_COLOUR.into(),
                    border: border::rounded(99.),
                },
            },
            ..scrollable::default(theme, status)
        })
}

/// Pad each element and align them vertically.
pub fn padded_column<'txt, Msg: 'txt, const N: usize>(
    elements: [Element<'txt, Msg>; N],
) -> Column<'txt, Msg> {
    Column::with_children(
        elements.into_iter().map(|elt| container(elt).padding(2.).into()),
    )
}
