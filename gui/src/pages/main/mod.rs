use crate::Page;
use crate::ui::component::txt;

/// Main page for one provider.
pub struct MainPage;

impl Page for MainPage {
    type Message = ();
    type Update = ();

    fn update(&mut self, (): Self::Message) -> Self::Update {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        txt("hi").into()
    }
}
