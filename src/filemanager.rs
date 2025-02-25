use std::{fs::create_dir_all, path::PathBuf};
use std::collections::HashMap;
use directories::ProjectDirs;
use std::fs::read_to_string;
use std::io::Write;
use std::fs::File;

use crate::music::Song;

/// Creates and then returns the path to a suitable location for application data to be stored.
/// If the path already exists, just return the path.
pub fn get_application_directory() -> Result<PathBuf, ()> {
    let project_dir = match ProjectDirs::from("com", "hchap1", "resonate") {
        Some(project_dir) => project_dir,
        None => return Err(())
    };

    let path = project_dir.data_dir().to_path_buf();
    let _ = create_dir_all(&path);
    Ok(path)
}

// This is a record of all touched songs, any time a search is made the results are added
pub struct SongCache {
    songs: Vec<Song>,
    metahash: HashMap<String, usize>, // hashmap relating the ID the idx of the song in the cache
    file: PathBuf
}

impl SongCache {

    fn readlines(&self) -> Vec<String> {
        match read_to_string(&self.file) {
            Ok(contents) => contents.lines().map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => Vec::<String>::new()
        }
    }

    fn rebuild_hashmap(&mut self) {
        self.metahash.clear();
        self.songs.iter().enumerate().for_each(|song| {
            self.metahash.insert(song.1.id.clone(), song.0);
        });
    }

    fn reload_cache(&mut self) {
        self.metahash.clear();
        self.songs = self.readlines().into_iter().enumerate().map(|line| {
            let song = Song::deserialise(line.1);
            self.metahash.insert(song.id.clone(), line.0);
            song
        }).collect::<Vec<Song>>();
    }

    fn rewrite_file(&self) {
        let mut file: File = File::create(&self.file).unwrap();
        self.songs.iter().for_each(|song| writeln!(file, "{}", song.serialise()).unwrap());
    }

    /// Load the SongCache from file. If no such file exists, it will be created
    pub fn load(directory: PathBuf) -> Self {
        let file = directory.join(PathBuf::from("songcache.txt"));
        if !file.exists() {
            let _ = File::create(&file);
        }

        let mut song_cache = Self {
            songs: Vec::<Song>::new(),
            file,
            metahash: HashMap::<String, usize>::new()
        };

        song_cache.reload_cache();
        song_cache
    }

    pub fn get_song_data_by_id(&self, id: &String) -> Option<Song> {
        match self.metahash.get(id) {
            Some(idx) => Some(self.songs[*idx].clone()),
            None => None
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.metahash.insert(song.id.clone(), self.songs.len());
        self.songs.push(song);
        self.rewrite_file();
    }

    pub fn remove_song(&mut self, song: Song) {
        let idx = match self.metahash.get(&song.id) {
            Some(idx) => idx.clone(),
            None => return
        };

        self.songs.remove(idx);
        self.rebuild_hashmap();
        self.rewrite_file();
    }
}
