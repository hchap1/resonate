use std::path::PathBuf;

use iced::widget::container;
use iced::widget::Column;
use iced::widget::Scrollable;
use iced::Background;
use iced::Color;
use iced::Task;
use iced::widget::button;
use iced::widget::Container;
use iced::Element;

use crate::downloader::Downloader;
use crate::filemanager::get_application_directory;
use crate::filemanager::Database;
use crate::music::{Song, local_search, cloud_search};
use crate::utility::*;
use crate::widgets::song_widget;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Message {
    Quit,
    Search(String),
    
    SearchResults(Vec<Song>)
}

// The underlying application state
#[derive(Default, Clone, Eq, PartialEq)]
pub enum State {
    #[default]
    Search
}

pub struct Application {

    // Frontend
    state: State,

    // Backends
    database: AM<Database>,
    downloader: AM<Downloader>,
    buffer: AMV<Song>
}

impl std::default::Default for Application {
    fn default() -> Self {
        let directory: PathBuf = get_application_directory().unwrap();
        Self::new(Database::new(directory.clone()), Downloader::new(directory))
    }
}

impl Application {
    pub fn new(database: Database, downloader: Downloader) -> Self {
        Self {
            state: State::default(),
            database: sync(database),
            downloader: sync(downloader),
            buffer: sync(vec![Song::example()])
        }
    }

    fn get_db_ref(&self) -> AM<Database> { self.database.clone() }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => iced::exit::<Message>(),

            Message::Search(q) => {
                println!("Searching: {q}");
                Task::<Message>::batch(vec![
                    Task::<Message>::future(local_search(q.clone(), self.get_db_ref())).map(|msg| msg),
                    Task::<Message>::future(cloud_search(q.clone(), self.get_db_ref())).map(|msg| msg)
                ])
            }

            Message::SearchResults(songs) => {
                let mut buf = self.buffer.lock().unwrap();
                songs.into_iter().for_each(|song| buf.push(song));
                Task::<Message>::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {

        let buf = self.buffer.lock().unwrap();
        let songs: Vec<Element<Message>> = buf
            .iter()
            .map(|song| song_widget(song.clone()))
            .collect();

        let mut widgets = Column::new().spacing(10).push(button("Seach 'COLDPLAY'").on_press(Message::Search(String::from("coldplay"))));
        let mut song_columns: Column<Message> = Column::new().spacing(10);
        for song in songs { song_columns = song_columns.push(song); }

        let scrollable_song_list: Scrollable<Message> = Scrollable::new(song_columns);
        widgets = widgets.push(scrollable_song_list);

        Container::new(widgets)
            .style(|_theme| {
                container::Style::default().background(
                    Background::Color(Color::from_rgb(0.1f32, 0.1f32, 0.1f32))
                )
            })
            .into()
    }
}
