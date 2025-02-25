use std::path::PathBuf;

const DELIM: char = '͵'; // Unicode 0372, greek lower numeral sign

#[derive(Clone)]
pub struct Song {
    pub name: String,
    pub artist: String,
    pub id: String,
    pub file: Option<PathBuf>
}

pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>
}

impl Song {
    pub fn new(name: String, artist: String, id: String, file: Option<PathBuf>) -> Self {
        Self { name, artist, id, file }
    }

    pub fn deserialise(serial: String) -> Self {
        // Format in cache is:
        // name|artist|id|file, if file is none then _ else absolute filepath
        let components = serial.split(DELIM).map(|x| x.to_string()).collect::<Vec<String>>();
        Self {
            name: components[0].clone(),
            artist: components[1].clone(),
            id: components[2].clone(),
            file: if &components[3] == "_" { None } else { Some(PathBuf::from(&components[3])) }
        }
    }

    pub fn serialise(&self) -> String {
        format!("{}{DELIM}{}{DELIM}{}{DELIM}{}",
            self.name,
            self.artist,
            self.id,
            match &self.file {
                Some(filepath) => filepath.to_string_lossy().to_string(),
                None => String::from("_")
            }
        )
    }
}
