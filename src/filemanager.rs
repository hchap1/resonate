use std::fs::create_dir_all;
use std::collections::HashSet;
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::{application::Message, music::Song};

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

pub struct Database {
    connection: Connection,
    directory: PathBuf
}

impl Database {
    pub fn new(directory: PathBuf) -> Self {
        let connection: Connection = match Connection::open(directory.join("data.db")) {
            Ok(connection) => connection,
            Err(_) => panic!("Could not create or access database file!")
        };

        // Ensure tables exist
        let _ = connection.execute("
            CREATE TABLE IF NOT EXISTS Songs (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                duration_s INT NOT NULL,
                downloaded INT NOT NULL
            );
        ",[]);

        Self { connection, directory }
    }

    pub fn add_song_to_cache(&self, song: &Song) {
        let _ = self.connection.execute(format!("
            INSERT INTO Songs
            VALUES('{}', '{}', '{}', '{}', {}, {});
        ", song.id, song.name, song.artist, song.album, song.duration, if song.file == None { 0 } else { 1 }
        ).as_str(),[]);
    }

    pub fn add_songs_to_cache(&self, songs: &Vec<Song>) {
        songs.iter().for_each(|song| self.add_song_to_cache(song));
    }

    pub fn retrieve_all_songs(&self) -> Vec<Song> {
        // ID, name, artist, album, duration, exists
        let mut pattern = self.connection.prepare("SELECT * FROM Songs").unwrap();
        pattern.query_map([], |row| {
            let id = row.get(0).unwrap();
            let file = match row.get(5).unwrap() {
                0 => None,
                _ => Some(self.directory.join(PathBuf::from(&id)))
            };
            Ok(Song::new(
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                id,
                row.get(3).unwrap(),
                row.get::<_, usize>(4).unwrap(),
                file
            ) )
        }).unwrap().map(|x| x.unwrap()).collect()
    }

    pub fn hash_all_songs(&self) -> HashSet<String> {
        let mut hash: HashSet<String> = HashSet::<String>::new();

        // ID, name, artist, album, duration, exists
        let mut pattern = self.connection.prepare("SELECT id FROM Songs").unwrap();
        pattern.query_map([], |row| {
            let id = row.get(0).unwrap();
            Ok(id)
        }).unwrap().for_each(|x| { hash.insert(x.unwrap()); });
        hash
    }

    pub fn search_cached_song(&self, query: String) -> Vec<Song> {
        let like_query = format!("%{query}%");
        let mut pattern = self.connection.prepare("SELECT * FROM Songs WHERE name LIKE ? OR artist LIKE ?").unwrap();
        pattern.query_map(params![like_query, like_query], |row| {
            Ok({
                let id = row.get::<_, String>(0).unwrap();
                let name = row.get::<_, String>(1).unwrap();
                let artist = row.get::<_, String>(2).unwrap();
                let album = row.get::<_, String>(3).unwrap();
                let duration_s = row.get::<_, usize>(4).unwrap();
                let downloaded = match row.get::<_, usize>(5) {
                    Ok(d) => if d == 0 { None } else { Some(self.directory.join(PathBuf::from(&id))) },
                    Err(_) => None
                };
                Song::new(name, artist, album, id, duration_s, downloaded)
            })
        }).unwrap().map(|x| x.unwrap()).collect::<Vec<Song>>()
    }

    pub fn get_directory(&self) -> PathBuf {
        self.directory.clone()
    }

    pub fn update(&self, song: Song) {
        let sql = "UPDATE Songs SET name = ?, artist = ?, album = ?, duration_s = ?, downloaded = ? WHERE id = ?";
        self.connection.execute(sql, params![song.name, song.artist, song.album, song.duration, match song.file { Some(_) => 1, None => 0 }]);
    }
}
