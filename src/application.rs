use std::collections::HashSet;
use std::path::PathBuf;

use iced::alignment::Horizontal;
use iced::widget::Row;
use iced::widget::Scrollable;
use iced::widget::Container;
use iced::widget::container;
use iced::widget::Column;
use iced::widget::button;
use iced::widget::text;
use iced::Background;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Border;
use iced::Theme;
use iced::Color;
use iced::Task;

use crate::music::{Song, local_search, cloud_search};
use crate::filemanager::get_application_directory;
use crate::widgets::display_song_widget;
use crate::widgets::playlist_name_widget;
use crate::widgets::download_song_widget;
use crate::widgets::playlist_search_bar;
use crate::widgets::playlist_widget;
use crate::widgets::ResonateColour;
use crate::filemanager::Database;
use crate::downloader::download;
use crate::widgets::search_bar;
use crate::music::Playlist;
use crate::utility::*;

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
    SearchPlaylists,
    NewPlaylist,
    CreateNewPlaylist,
    OpenPlaylist(Playlist),
    AddSongs,
    Homepage
}

// The underlying application state

#[derive(Default, Clone, Eq, PartialEq)]
pub enum State {
    #[default]
    SearchPlaylists,
    Search,
    MakePlaylist,
    Playlist
}

pub struct Application {

    // Frontend
    state: State,

    // Backends
    database: AM<Database>,
    buffer: AMV<Song>,
    playlist_buffer: Vec<Playlist>,
    search_bar: String,
    
    active_search_threads: usize,
    use_online_search: bool,

    currently_download_songs: HashSet<Song>,
    download_queue: Vec<Song>,

    // Targetted playlist
    target_playlist: Option<Playlist>,
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
            download_queue: Vec::<Song>::new(),
            target_playlist: None,
            playlist_buffer: Vec::new()
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

                println!("DOWNLOAD REQUEST FOR: {}", s.name);

                if self.currently_download_songs.contains(&s) || s.file.is_some() || self.download_queue.contains(&s) {
                    if s.file.is_some() {
                        let database = self.database.lock().unwrap();
                        database.add_song_to_playlist(&s, &mut self.target_playlist.as_mut().unwrap());
                        println!("Added to playlist.");
                    }
                    println!("Refused request.");
                    return Task::none()
                }

                if self.currently_download_songs.len() >= 4 {
                    if !self.download_queue.contains(&s) { self.download_queue.push(s); }
                    println!("Queued, workers occupied.");
                    Task::none()
                } else {
                    self.currently_download_songs.insert(s.clone());
                    println!("Downloading.");
                    Task::future(download(d, s)).map(|msg| msg)
                }
            }

            // When a song is successfully downloaded, update the database and redraw
            Message::SuccessfulDownload(song) => {
                println!("[RUNTIME] Received successful download of {}", song.name);
                let mut songs_to_remove: Vec<Song> = Vec::new();
                self.currently_download_songs.iter().for_each(|song| if song.sql_id == song.sql_id { songs_to_remove.push(song.clone()); });
                for song in songs_to_remove {
                    println!("Dequeued {}", song.name);
                    self.currently_download_songs.remove(&song);
                }

                // Update song view
                let mut buf = self.buffer.lock().unwrap();
                for s in buf.iter_mut() {
                    if s.id == song.id {
                        println!("[RUNTIME] Updated current view of {}", song.name);
                        s.file = song.file.clone();
                    }
                }
                let database = self.database.lock().unwrap();
                println!("[RUNTIME] About to add {}. Is_some: {}.", song.name, self.target_playlist.is_some());
                database.add_song_to_playlist(&song, &mut self.target_playlist.as_mut().unwrap());
                database.update(song);
                let directory = database.get_directory();
                
                if self.download_queue.is_empty() { Task::none() } else {
                    let song = self.download_queue.remove(0);
                    self.currently_download_songs.insert(song.clone());
                    Task::future(download(directory, song))
                }
            }

            Message::SearchPlaylists => {
                println!("[RUNTIME] Searching {}", self.search_bar);
                let database = self.database.lock().unwrap();
                self.playlist_buffer = 
                    if self.search_bar.len() > 0 { database.search_playlist_by_name(self.search_bar.clone()) } 
                    else { database.dump_all_playlists() };
                self.search_bar.clear();
                Task::none()
            }

            Message::NewPlaylist => {
                self.state = State::MakePlaylist;
                self.search_bar.clear();
                self.playlist_buffer.clear();
                Task::none()
            }

            Message::CreateNewPlaylist => {
                let database = self.database.lock().unwrap();
                database.create_playlist(self.search_bar.clone());
                self.search_bar.clear();
                self.state = State::SearchPlaylists;
                Task::none()
            }

            Message::Homepage => {
                self.search_bar.clear();
                self.state = State::SearchPlaylists;
                let database = self.database.lock().unwrap();
                self.playlist_buffer = database.dump_all_playlists();
                Task::none()
            }

            Message::OpenPlaylist(p) => {
                let mut buf = self.buffer.lock().unwrap();
                buf.clear();
                let database = self.database.lock().unwrap();
                let mut playlist = p.clone();
                database.load_playlist(&mut playlist);
                playlist.songs.as_ref().unwrap().iter().for_each(|song| buf.push(song.clone()));
                self.target_playlist = Some(playlist);
                self.state = State::Playlist;
                Task::none()
            }

            Message::AddSongs => {
                self.state = State::Search;
                self.search_bar.clear();
                let mut buf = self.buffer.lock().unwrap();
                buf.clear();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {

        let widgets = match self.state {
            State::SearchPlaylists => {
                let widgets = Column::new()
                    .spacing(10)
                    .push(playlist_search_bar(String::from("Search..."), &self.search_bar));

                let mut playlist_list = Column::new().spacing(10);

                let p = 
                    if self.playlist_buffer.len() > 0 { self.playlist_buffer.clone() }
                    else { let db = self.database.lock().unwrap(); db.dump_all_playlists() };

                for playlist in p.into_iter() {
                    playlist_list = playlist_list.push(playlist_widget(playlist.clone()))
                }

                widgets.push(Scrollable::new(playlist_list))
            }

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
                        let is_queued = !is_downloading && self.download_queue.contains(&song);
                        download_song_widget(song.clone(), dir.clone(), is_downloading, is_queued)
                    })
                    .collect();

                let name = match &self.target_playlist {
                    Some(playlist) => playlist.name.clone(),
                    None => String::from("NO PLAYLIST")
                };

                let playlist = self.target_playlist.as_ref().unwrap().clone();

                let widgets = Column::new()
                    .spacing(10)
                    .push(text(name).size(50).color(ResonateColour::text_emphasis()))
                    .push(button("Back to Playlist")
                        .on_press(Message::OpenPlaylist(playlist)))
                    .push(search_bar("Search...".to_string(), &self.search_bar, self.use_online_search));

                let mut song_columns: Column<Message> = Column::new().spacing(10);
                for song in songs { song_columns = song_columns.push(song); }

                let scrollable_song_list: Scrollable<Message> = Scrollable::new(song_columns);
                widgets.push(scrollable_song_list)

            }

            State::MakePlaylist => {
                Column::new()
                    .push(playlist_name_widget(String::from("Enter Playlist Name"), &self.search_bar))
            }

            State::Playlist => {
                let name = match &self.target_playlist {
                    Some(playlist) => playlist.name.clone(),
                    None => String::from("404 - Braincell not found.")
                };

                let widgets = Column::new()
                    .spacing(10)
                    .width(Length::Fill)
                    .push(text(name).size(50).color(ResonateColour::text_emphasis()))
                    .push(Row::new().spacing(10).push(
                        button("Add Songs")
                        .style(|_theme: &Theme, style| button::Style {
                            background: match style {
                                button::Status::Hovered => Some(Background::Color(ResonateColour::darken(ResonateColour::blue()))),
                                _ => Some(Background::Color(ResonateColour::blue()))
                            },
                            border: Border::default().rounded(10),
                            shadow: Shadow::default(),
                            text_color: ResonateColour::text(),
                        })
                        .on_press(Message::AddSongs)
                    )
                    .push(
                        button("Home")
                        .style(|_theme: &Theme, style| button::Style {
                            background: match style {
                                button::Status::Hovered => Some(Background::Color(ResonateColour::darken(ResonateColour::blue()))),
                                _ => Some(Background::Color(ResonateColour::blue()))
                            },
                            border: Border::default().rounded(10),
                            shadow: Shadow::default(),
                            text_color: ResonateColour::text(),
                        })
                        .on_press(Message::Homepage)
                    ));

                let buf = self.buffer.lock().unwrap();
                let songs: Vec<Element<Message>> = buf
                    .iter()
                    .map(|song| {
                        display_song_widget(song.clone())
                    })
                    .collect();

                let mut song_columns: Column<Message> = Column::new().spacing(10);
                for song in songs { song_columns = song_columns.push(song); }

                let scrollable_song_list: Scrollable<Message> = Scrollable::new(song_columns);
                widgets.push(scrollable_song_list)
            }
        };
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
