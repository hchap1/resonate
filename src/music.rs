use std::path::PathBuf;

pub struct Song {
    pub name: String,
    pub artist: String,
    pub id: String,
    pub file: Option<PathBuf>
}

impl Song {
    pub fn new(name: String, artist: String, id: String, file: Option<PathBuf>) -> Self {
        Self { name, artist, id, file }
    }
}
