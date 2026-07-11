use iced::border::{self, rounded};
use iced::widget::text::IntoFragment;
use iced::widget::{Button, Text, TextInput, button, text, text_input};
use iced::{Element, Font, Renderer, Theme};

use crate::ui::style::{
    BORDER_WIDTH, BTN_COLOUR, FOCUSED_COLOUR, RADIUS, TXT_COLOUR, TXT_FONT, grey
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
) -> TextInput<'msg, Msg> {
    use text_input::Status as St;
    text_input(placeholder, value)
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
) -> Button<'disp, Msg> {
    button(content).on_press(msg).style(|_, st| button::Style {
        background: Some(
            if matches!(st, button::Status::Active) {
                BTN_COLOUR
            } else {
                FOCUSED_COLOUR
            }
            .into(),
        ),
        border: rounded(RADIUS),
        ..Default::default()
    })
}

/// Displays a simple piece of text.
pub fn txt<'txt, Content: IntoFragment<'txt>>(
    content: Content,
) -> Text<'txt, Theme, Renderer> {
    text(content).font(Font::MONOSPACE).size(TXT_FONT).color(TXT_COLOUR)
}
