//! GUI mailbox application.

use core::mem::take;
use std::sync::{Mutex, PoisonError};

use color_eyre::Result;
use iced::widget::container::Style;
use iced::widget::{Column, button, container, text};
use iced::{Color, Element, Length, Task};
use mailbox_shared::Config;

/// Gui Application state.
#[non_exhaustive]
pub struct GuiApp(Config);

impl GuiApp {
    /// Runs the gui application.
    ///
    /// # Errors
    ///
    /// Returns an error if the rendering or configuration loading fails.
    pub fn run() -> Result<()> {
        let config = Mutex::new(Config::load()?);
        let boot = move || {
            Self(take(
                &mut config.lock().unwrap_or_else(PoisonError::into_inner),
            ))
        };
        let app = iced::application(boot, Self::update, Self::view);
        app.run()?;
        Ok(())
    }

    /// Updates the application based on incomming messages.
    fn update(&mut self, (): ()) -> Task<()> {
        Task::none()
    }

    /// Displays the app.
    fn view(&self) -> Element<'_, ()> {
        let mut layout = Column::new();
        layout = layout.push(text("hi").color(Color::WHITE));
        layout = layout.push(button("Click Me!").on_press(()));

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| Style::default().background(Color::BLACK))
            .into()
    }
}
