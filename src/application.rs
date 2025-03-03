use std::path::PathBuf;

use iced::Task;
use iced::widget::button;
use iced::widget::text;
use iced::widget::column;
use iced::widget::center;
use iced::Element;

use crate::downloader::Downloader;
use crate::filemanager::get_application_directory;
use crate::filemanager::Database;
use crate::music::{SearchTask, Song};
use crate::utility::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Message {
    Quit,
    Search(String),
    
    IncomingSearch(Song)
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
                Task::<Message>::stream(SearchTask::new(q, self.get_db_ref()))
            }

            Message::IncomingSearch(s) => {
                let mut buf = self.buffer.lock().unwrap();
                buf.push(s);
                println!("Received message to add song!");
                Task::<Message>::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut widgets = column![button("Seach 'COLDPLAY'").on_press(Message::Search(String::from("coldplay")))];

        let buf = self.buffer.lock().unwrap();

        for s in buf.iter() {
            widgets = widgets.push(text(s.display()));
        }

        center(widgets).padding(20).into()
    }
}
