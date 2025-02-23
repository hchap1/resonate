use std::path::PathBuf;

pub struct Song {
    name: String,
    artist: String,
    url: String,
    file: Option<PathBuf>
}

impl Song {
    pub fn new(name: String, artist: String, url: String, file: Option<PathBuf>) -> Self {
        Self { name, artist, url, file }
    }
}
