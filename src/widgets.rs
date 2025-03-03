use iced::{alignment::Vertical, widget::{container, text, Column, Container, Row}, Background, Color, Element, Length, Theme};

use crate::{application::Message, music::Song};

pub fn song_widget(song: Song) -> Element<'static, Message> {
    let title = text(song.name).color(Color::WHITE).size(25);
    let artist = text(song.artist).color(Color::from_rgb(0.7f32, 0.7f32, 0.7f32));
    let album = text(song.album).color(Color::from_rgb(0.7f32, 0.7f32, 0.7f32));
    let duration = text(song.duration);

    let row = Row::new()
        .spacing(30)
        .push(
            Column::new()
                .push(title)
                .push(artist)
                .width(Length::FillPortion(1))
        )
        .push(album.width(Length::FillPortion(1)))
        .push(duration.width(Length::FillPortion(1)))
        .align_y(Vertical::Center);

    Container::new(row)
        .padding(20)
        .width(Length::Fill)
        .center_y(Length::Fill)
        .height(Length::Shrink)
        .style(|_theme: &Theme| {
            container::Style::default().background(Background::Color(Color::from_rgb(0.4f32, 0.4f32, 0.4f32)))
        })
        .into()
}
