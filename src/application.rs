use std::path::PathBuf;

use iced::widget::Column;
use iced::Task;
use iced::widget::button;
use iced::widget::text;
use iced::widget::column;
use iced::widget::center;
use iced::Element;

use crate::downloader::Downloader;
use crate::filemanager::get_application_directory;
use crate::filemanager::Database;
use crate::music::{search_and_dump, Song};
use crate::utility::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Message {
    Quit,
    Search(String)
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
            buffer: sync(Vec::<Song>::new())
        }
    }

    fn get_db_ref(&self) -> AM<Database> { self.database.clone() }
    fn get_buf_ref(&self) -> AMV<Song> { self.buffer.clone() }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => iced::exit::<Message>(),
            Message::Search(q) => {
                let db = self.get_db_ref();
                let buf = self.get_buf_ref();
                Task::<Message>::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let widgets = column![button("Seach 'COLDPLAY'").on_press(Message::Search(String::from("coldplay")))];
        center(widgets).padding(20).into()
    }
}
