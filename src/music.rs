use std::path::PathBuf;

use crate::filemanager::Database;
use crate::downloader::search_youtube_music;
use crate::utility::*;

const DELIM: char = '͵'; // Unicode 0372, greek lower numeral sign

#[derive(Clone)]
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
}

/// Searches database and youtube music, inserting all songs into database and pushing them as they arrive into an buffer
pub async fn search_and_dump(query: String, database: AM<Database>, dump: AMV<Song>) {
    let (directory, db_hash) = {
        let database = database.lock().unwrap();
        let mut dump = dump.lock().unwrap();
        // Immediately return cached songs
        dump.append(&mut database.search_cached_song(query.clone()));
        (database.get_directory(), database.hash_all_songs())
    };
    
    let mut results: Vec<Song> = search_youtube_music(query, directory).await.unwrap().into_iter().filter(|song| db_hash.contains(&song.id)).collect();
    let mut dump = dump.lock().unwrap();
    dump.append(&mut results);
}
