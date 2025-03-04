use std::path::PathBuf;

use iced::widget::container;
use iced::widget::text;
use iced::widget::Column;
use iced::widget::Scrollable;
use iced::Background;
use iced::Color;
use iced::Length;
use iced::Task;
use iced::widget::Container;
use iced::Element;
use std::collections::HashSet;

use crate::downloader::download;
use crate::filemanager::get_application_directory;
use crate::filemanager::Database;
use crate::music::{Song, local_search, cloud_search};
use crate::utility::*;
use crate::widgets::search_bar;
use crate::widgets::song_widget;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Message {
    Quit,
    Search,
    SearchBarInput(String),
    SearchResults(Vec<Song>),
    DumpDB,
    ToggleYTSearch(bool),
    Download(Song, PathBuf),
    SuccessfulDownload(Song),
    Downloading(Song)
}

// The underlying application state

#[derive(Default, Clone, Eq, PartialEq)]
pub enum State {
    #[default]
    Search,
}

pub struct Application {

    // Frontend
    state: State,

    // Backends
    database: AM<Database>,
    buffer: AMV<Song>,
    search_bar: String,
    
    active_search_threads: usize,
    use_online_search: bool,

    currently_download_songs: HashSet<Song>,
    download_queue: Vec<Song>
}

impl std::default::Default for Application {
    fn default() -> Self {
        let directory: PathBuf = get_application_directory().unwrap();
        Self::new(Database::new(directory.clone()))
    }
}

impl Application {
    pub fn new(database: Database) -> Self {
        Self {
            state: State::default(),
            database: sync(database),
            buffer: sync(vec![]),
            search_bar: String::new(),
            active_search_threads: 0,
            use_online_search: false,
            currently_download_songs: HashSet::<Song>::new(),
            download_queue: Vec::<Song>::new()
        }
    }

    fn get_db_ref(&self) -> AM<Database> { self.database.clone() }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => iced::exit::<Message>(),

            Message::Search => {
                if self.search_bar.len() == 0 {
                    let mut buf = self.buffer.lock().unwrap();
                    buf.clear();
                    Task::none()
                } else {
                    if self.active_search_threads != 0 { return Task::none() }
                    let mut buf = self.buffer.lock().unwrap();
                    buf.clear();
                    let mut tasks: Vec<Task<Message>> = Vec::new();
                    tasks.push(Task::<Message>::future(local_search(self.search_bar.clone(), self.get_db_ref())).map(|msg| msg));
                    if self.use_online_search { tasks.push(Task::<Message>::future(cloud_search(self.search_bar.clone(), self.get_db_ref())).map(|msg| msg)) }
                    let task = Task::<Message>::batch(tasks);
                    self.search_bar.clear();
                    if self.use_online_search { self.active_search_threads = 2; }
                    else { self.active_search_threads = 1; }
                    task
                }
            }

            Message::SearchResults(songs) => {
                let mut buf = self.buffer.lock().unwrap();
                songs.into_iter().for_each(|song| buf.push(song));
                if self.active_search_threads > 0 { self.active_search_threads -= 1; }
                Task::<Message>::none()
            }

            Message::SearchBarInput(s) => {
                self.search_bar = s;
                Task::none()
            }

            Message::DumpDB => {
                let mut buf = self.buffer.lock().unwrap();
                buf.clear();
                let database = self.database.lock().unwrap();
                database.retrieve_all_songs().into_iter().for_each(|song| buf.push(song));
                Task::none()
            }

            Message::ToggleYTSearch(b) => {
                self.use_online_search = b;
                Task::none()
            }

            Message::Download(s, d) => {

                if self.currently_download_songs.contains(&s) {
                    return Task::none()
                }

                if self.currently_download_songs.len() >= 4 {
                    if !self.download_queue.contains(&s) { self.download_queue.push(s); }
                    Task::none()
                } else {
                    Task::future(download(d, s)).map(|msg| msg)
                }
            }

            // When a song is successfully downloaded, update the database and redraw
            Message::SuccessfulDownload(song) => {
                println!("[RUNTIME] Received successful download of {}", song.name);
                self.currently_download_songs.remove(&song);
                
                // Update song view
                let mut buf = self.buffer.lock().unwrap();
                for s in buf.iter_mut() {
                    if s.id == song.id {
                        s.file = song.file.clone()
                    }
                }
                let database = self.database.lock().unwrap();
                database.update(song);
                let directory = database.get_directory();

                if self.download_queue.is_empty() { Task::none() } else { Task::future(download(directory, self.download_queue.remove(0))) }
            }

            Message::Downloading(song) => {
                self.currently_download_songs.insert(song);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {

        match self.state {
            State::Search => {
                let buf = self.buffer.lock().unwrap();
                let dir = {
                    let db = self.database.lock().unwrap();
                    db.get_directory()
                };
                let songs: Vec<Element<Message>> = buf
                    .iter()
                    .map(|song| {
                        let is_downloading = self.currently_download_songs.contains(&song);
                        song_widget(song.clone(), dir.clone(), is_downloading)
                    })
                    .collect();

                let mut tasks_col = Column::new().push(text("CURRENT DOWNLOAD TASKS"));

                for song in self.currently_download_songs.iter() {
                    tasks_col = tasks_col.push(text(song.name.clone()));
                }

                let mut widgets = Column::new()
                    .spacing(10)
                    .push(search_bar("Search ...".to_string(), &self.search_bar, self.use_online_search))
                    .push(tasks_col);

                let mut song_columns: Column<Message> = Column::new().spacing(10);
                for song in songs { song_columns = song_columns.push(song); }

                let scrollable_song_list: Scrollable<Message> = Scrollable::new(song_columns);
                widgets = widgets.push(scrollable_song_list);

                Container::new(widgets)
                    .padding(20)
                    .style(|_theme| {
                        container::Style::default().background(
                            Background::Color(Color::from_rgb(0.1f32, 0.1f32, 0.1f32))
                        )
                    })
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
        }
    }
}
