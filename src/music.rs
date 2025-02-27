use std::path::PathBuf;

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
