use std::path::PathBuf;

use crate::application::Message;
use crate::filemanager::Database;
use crate::downloader::search_youtube_music;
use crate::utility::*;

const DELIM: char = '͵'; // Unicode 0372, greek lower numeral sign

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Song {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub id: String,
    pub duration: usize,
    pub file: Option<PathBuf>
}

pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>
}

impl Song {
    pub fn new(name: String, artist: String, album: String, id: String, duration: usize, file: Option<PathBuf>) -> Self {
        Self { name, artist, album, id, duration, file }
    }

    pub fn example() -> Self {
        Self::new(
            String::from("TestSong"),
            String::from("hchap1"),
            String::from("Examples"),
            String::from("123abc"),
            10,
            None
        )
    }

    pub fn display(&self) -> String {
        format!(
            "{} by {} in {}. {} seconds, ID: {}. File: {}",
            self.name, self.artist, self.album, self.duration, self.id,
            match &self.file {
                Some(path) => path.to_string_lossy().to_string(),
                None => "Not downloaded.".to_string()
            }
        )
    }
}

pub async fn local_search(query: String, database: AM<Database>) -> Message {
    println!("[LOCAL] Thread started.");
    let database = database.lock().unwrap();
    Message::SearchResults(database.search_cached_song(query))
}

pub async fn cloud_search(query: String, database: AM<Database>) -> Message {
    println!("[CLOUD] Thread started.");

    // Find directory and get a HashSet of all cached songs then implicitly drop mutex
    let (directory, db_hash) = {
        let database = database.lock().unwrap();
        (database.get_directory(), database.hash_all_songs())
    };

    // Async blocking call to search youtube music, this should take about 5-10 seconds
    let results: Vec<Song> = tokio::task::spawn_blocking(move || {
        search_youtube_music(query, directory)
            .unwrap()
            .into_iter()
            .filter(|song| !db_hash.contains(&song.id))
            .collect()
    }).await.unwrap();

    // Relock the mutex in order to cache the new songs
    let database = database.lock().unwrap();
    database.add_songs_to_cache(&results);
    Message::SearchResults(results)
}
