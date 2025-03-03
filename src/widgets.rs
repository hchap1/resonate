use iced::{widget::{text, Container, Row}, Color, Element, Length};

use crate::{application::Message, music::Song};

pub fn song_widget<'a>(song: Song) -> Element<'a, Message> {
    let title = text(song.name).color(Color::from_rgb(50f32, 0f32, 0f32));
    let artist = text(song.artist);
    let duration = text(song.duration);

    let row = Row::new()
        .spacing(10)
        .push(title)
        .push(artist)
        .push(duration);

    Container::new(row)
        .padding(20)
        .center(Length::Fill)
        .into()
}
