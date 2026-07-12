use iced::{Color, Font};

/// Default monospace font.
pub const FONT: Font = Font::MONOSPACE;

/// Default text font size.
pub const TXT_FONT: u32 = 8;

/// Default color of text.
pub const TXT_COLOUR: Color = Color::from_rgb8(255, 255, 220);

/// Colour of unfocused button.
pub const BTN_COLOUR: Color = Color::from_rgb8(50, 50, 100);

/// Yellow colour.
pub const YELLOW: Color = Color::from_rgb8(0xe5, 0xc0, 0x7b);

/// Red colour.
pub const RED: Color = Color::from_rgb8(0xe0, 0x6c, 0x75);

/// Colour of unfocused button.
pub const FOCUSED_COLOUR: Color = Color::from_rgb8(75, 75, 150);

/// Link colour.
pub const LINK_COLOUR: Color = Color::from_rgb8(100, 100, 200);

/// Unseen colour.
pub const UNSEEN_COLOUR: Color = Color::from_rgb8(200, 200, 255);

/// Radius of borders.
pub const RADIUS: f32 = 4.;

/// Default border width.
pub const BORDER_WIDTH: f32 = 0.5;

/// Returns a grey variant.
pub const fn grey(darkness: u8) -> Color {
    Color::from_rgb8(darkness, darkness, darkness)
}
