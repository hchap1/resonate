use iced::alignment::Vertical;

pub struct ResonateColour;
impl ResonateColour {

    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgb(r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32)
    }

    pub fn background() -> Color { Self::new(25, 25, 25) }
    pub fn foreground() -> Color { Self::new(35, 35, 35) }
    pub fn text_emphasis() -> Color { Self::new(200, 200, 200) }
    pub fn text() -> Color { Self::new(120, 120, 120) }
}

use iced::widget::{
    container, text, text_input, toggler, Column, Container, Row
};

use iced::{
    Background,
    Border,
    Color,
    Element,
    Length,
    Theme
};

use crate::{application::Message, music::Song};

pub fn song_widget(song: Song) -> Element<'static, Message> {
    let title = text(song.name).color(ResonateColour::text_emphasis()).size(25);
    let artist = text(song.artist).color(ResonateColour::text());
    let album = text(song.album).color(ResonateColour::text());
    let duration = text(format!("{} seconds", song.duration)).color(ResonateColour::text());

    let row = Row::new()
        .spacing(30)
        .push(
            Column::new()
                .push(title)
                .push(artist)
                .width(Length::FillPortion(4))
        )
        .push(album.width(Length::FillPortion(4)))
        .push(duration.width(Length::FillPortion(1)))
        .align_y(Vertical::Center);

    Container::new(row)
        .padding(20)
        .width(Length::Fill)
        .center_y(Length::Fill)
        .height(Length::Shrink)
        .style(|_theme: &Theme| {
            container::Style::default()
                .background(Background::Color(Color::from_rgb(0.15f32, 0.15f32, 0.15f32)))
                .border(Border::default().rounded(15))
        })
        .into()
}

pub fn search_bar(prompt: String, content: &String, toggle: bool) -> Element<'static, Message> {
    let widget = Row::new()
        .push(text_input(prompt.as_str(), content)
            .on_input(Message::SearchBarInput)
            .on_submit(Message::Search))
        .push(toggler(toggle)
        .on_toggle(Message::ToggleYTSearch));
    
    Container::new(widget)
        .padding(20)
        .width(Length::Fill)
        .style(|_theme: &Theme| {
            container::Style::default()
                .background(Background::Color(ResonateColour::foreground()))
                .border(Border::default().rounded(15))
        })
        .into()
}
