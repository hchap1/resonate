use std::path::PathBuf;

use crate::application::Message;
use crate::filemanager::Database;
use crate::downloader::search_youtube_music;
use crate::utility::*;

const DELIM: char = '͵'; // Unicode 0372, greek lower numeral sign

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Song {
    pub sql_id: usize,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub id: String,
    pub duration: usize,
    pub file: Option<PathBuf>
}

impl Song {
    pub fn new(sql_id: usize, name: String, artist: String, album: String, id: String, duration: usize, file: Option<PathBuf>) -> Self {
        Self { sql_id, name, artist, album, id, duration, file }
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
    let mut results: Vec<Song> = tokio::task::spawn_blocking(move || {
        search_youtube_music(query, directory)
            .unwrap()
            .into_iter()
            .filter(|song| !db_hash.contains(&song.id))
            .collect()
    }).await.unwrap();

    // Relock the mutex in order to cache the new songs
    let database = database.lock().unwrap();
    database.add_songs_to_cache(&mut results);
    Message::SearchResults(results)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playlist {
    pub id: usize,
    pub name: String,
    pub songs: Option<Vec<Song>>
}
