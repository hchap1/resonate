use std::path::PathBuf;

use iced::alignment::Vertical;

pub struct ResonateColour;
impl ResonateColour {

    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgb(r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32)
    }

    pub fn background() -> Color { Self::new(25, 25, 25) }
    pub fn foreground() -> Color { Self::new(35, 35, 35) }
    pub fn accent() -> Color { Self::new(55, 55, 55) }
    pub fn text_emphasis() -> Color { Self::new(200, 200, 200) }
    pub fn text() -> Color { Self::new(120, 120, 120) }
    pub fn green() -> Color { Self::new(20, 140, 20) }
    pub fn red() -> Color { Self::new(140, 20, 20) }
    pub fn blue() -> Color { Self::new(9, 84, 141) }

    pub fn darken(color: Color) -> Color { Color::from_rgb(color.r * 0.8, color.g * 0.8, color.b * 0.8) }
}

use iced::widget::{
    button, container, text, text_input, toggler, Column, Container, Row
};

use iced::{
    Background, Border, Color, Element, Length, Shadow, Theme
};

use crate::{application::Message, music::Song};

pub fn song_widget(song: Song, directory: PathBuf) -> Element<'static, Message> {
    let song_clone = song.clone();

    let play_button = button("Download")
        .style(|_theme: &Theme, style| button::Style {
            background: match style {
                button::Status::Hovered => Some(Background::Color(ResonateColour::darken(ResonateColour::blue()))),
                _ => Some(Background::Color(ResonateColour::blue()))
            },
            border: Border::default().rounded(10),
            shadow: Shadow::default(),
            text_color: ResonateColour::text(),
        })
        .on_press_with(move || Message::Download(song_clone.clone(), directory.clone()));

    let downloaded = text(match song.file {
        Some(p) => p.to_string_lossy().to_string(),
        None => String::from("Not downloaded.")
    });
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
        .push(downloaded.width(Length::FillPortion(3)))
        .push(play_button.width(Length::FillPortion(1)))
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
        .spacing(20)
        .push(text_input(prompt.as_str(), content)
            .on_input(Message::SearchBarInput)
            .on_submit(Message::Search)
            .width(Length::FillPortion(5))
            .style(|_theme: &Theme, _style| text_input::Style {
                background: Background::Color(ResonateColour::accent()),
                border: Border::default().rounded(10),
                icon: ResonateColour::accent(),
                placeholder: ResonateColour::text(),
                value: ResonateColour::text_emphasis(),
                selection: ResonateColour::red()
            }))
        .push(toggler(toggle)
            .size(30)
            .on_toggle(Message::ToggleYTSearch)
            .style(move |_theme: &Theme, _style| toggler::Style {
                background: ResonateColour::background(),
                background_border_width: 0f32,
                background_border_color: ResonateColour::background(),
                foreground: if toggle { ResonateColour::green() } else { ResonateColour::red() },
                foreground_border_width: 0f32,
                foreground_border_color: ResonateColour::foreground()
            })
            .width(Length::FillPortion(1))
        )
        .align_y(Vertical::Center);
    
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
