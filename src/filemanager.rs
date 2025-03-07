use std::fs::create_dir_all;
use std::collections::HashSet;
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::music::{Playlist, Song};

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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ytid TEXT NOT NULL,
                name TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                duration_s INT NOT NULL,
                downloaded INT NOT NULL
            );
        ",[]);

        let _ = connection.execute("
            CREATE TABLE IF NOT EXISTS Playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );
        ",[]);

        let _ = connection.execute("
            CREATE TABLE Contents (
                playlist_id INTEGER,
                song_id INTEGER,
                PRIMARY KEY (playlist_id, song_id),
                FOREIGN KEY (playlist_id) REFERENCES Playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (song_id) REFERENCES Songs(id) ON DELETE CASCADE
            );
        ",[]);

        Self { connection, directory }
    }

    pub fn add_songs_to_cache(&self, songs: &mut Vec<Song>) {
        songs.iter_mut().for_each(|mut song| self.add_song_to_cache(&mut song));
    }

    pub fn retrieve_all_songs(&self) -> Vec<Song> {
        // ID, name, artist, album, duration, exists
        let mut pattern = self.connection.prepare("SELECT * FROM Songs").unwrap();
        pattern.query_map([], |row| {
            let id = row.get(1).unwrap();
            let file = match row.get(6).unwrap() {
                0 => None,
                _ => Some(self.directory.join(PathBuf::from(format!("{id}.mp3"))))
            };
            Ok(Song::new(
                row.get::<_, usize>(0).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                id,
                row.get(4).unwrap(),
                row.get::<_, usize>(5).unwrap(),
                file
            ))
        }).unwrap().map(|x| x.unwrap()).collect()
    }

    pub fn hash_all_songs(&self) -> HashSet<String> {
        let mut hash: HashSet<String> = HashSet::<String>::new();

        // ID, name, artist, album, duration, exists
        let mut pattern = self.connection.prepare("SELECT ytid FROM Songs").unwrap();
        pattern.query_map([], |row| {
            let id = row.get::<_, String>(0).unwrap();
            Ok(id)
        }).unwrap().for_each(|x| { hash.insert(x.unwrap()); });
        hash
    }

    pub fn search_cached_song(&self, query: String) -> Vec<Song> {
        let like_query = format!("%{query}%");
        let mut pattern = self.connection.prepare("SELECT * FROM Songs WHERE name LIKE ? OR artist LIKE ?").unwrap();
        pattern.query_map(params![like_query, like_query], |row| {
            Ok({
                let sql_id = row.get::<_, usize>(0).unwrap();
                let id = row.get::<_, String>(1).unwrap();
                let name = row.get::<_, String>(2).unwrap();
                let artist = row.get::<_, String>(3).unwrap();
                let album = row.get::<_, String>(4).unwrap();
                let duration_s = row.get::<_, usize>(5).unwrap();
                let downloaded = match row.get::<_, usize>(6) {
                    Ok(d) => if d == 0 { None } else { Some(self.directory.join(PathBuf::from(format!("{}.mp3", id)))) },
                    Err(_) => None
                };
                Song::new(sql_id, name, artist, album, id, duration_s, downloaded)
            })
        }).unwrap().map(|x| x.unwrap()).collect::<Vec<Song>>()
    }

    pub fn get_directory(&self) -> PathBuf {
        self.directory.clone()
    }

    pub fn update(&self, song: Song) {
        let sql = "UPDATE Songs SET downloaded = ? WHERE id = ?";
        let _ = self.connection.execute(sql, params![match song.file { Some(_) => 1, None => 0 }, song.sql_id]);
    }

    pub fn load_song_by_id(&self, id: usize) -> Song {
        let mut pattern = self.connection.prepare("SELECT * FROM Songs WHERE id = ?").unwrap();
        pattern.query_map(params![id], |row| {
            let sql_id = row.get::<_, usize>(0).unwrap();
            let id = row.get::<_, String>(1).unwrap();
            let name = row.get::<_, String>(2).unwrap();
            let artist = row.get::<_, String>(3).unwrap();
            let album = row.get::<_, String>(4).unwrap();
            let duration_s = row.get::<_, usize>(5).unwrap();
            let downloaded = match row.get::<_, usize>(6) {
                Ok(d) => if d == 0 { None } else { Some(self.directory.join(PathBuf::from(format!("{id}.mp3")))) },
                Err(_) => None
            };
            Ok(Song::new(sql_id, name, artist, album, id, duration_s, downloaded))
        }).unwrap().map(|x| x.unwrap()).collect::<Vec<Song>>().remove(0)
    }

    pub fn load_playlist(&self, playlist: &mut Playlist) {
        let mut pattern = self.connection.prepare("SELECT song_id FROM Contents WHERE playlist_id = ?").unwrap();
        playlist.songs = Some(
            pattern.query_map(params![playlist.id], |row| {
                Ok(self.load_song_by_id(row.get::<_, usize>(0).unwrap()))
            }).unwrap().map(|x| x.unwrap()).collect::<Vec<Song>>()
        );
    }

    pub fn search_playlist_by_name(&self, query: String) -> Vec<Playlist> {
        let like_query = format!("%{query}");
        let mut pattern = self.connection.prepare("SELECT * FROM Playlists WHERE name LIKE ?").unwrap();
        pattern.query_map(params![like_query], |row| {
            Ok(Playlist {
                id: row.get::<_, usize>(0).unwrap(),
                name: row.get::<_, String>(1).unwrap(),
                songs: None
            })
        }).unwrap().map(|playlist| playlist.unwrap()).collect()
    }

    pub fn dump_all_playlists(&self) -> Vec<Playlist> {
        let mut pattern = self.connection.prepare("SELECT * FROM Playlists").unwrap();
        let mut playlists = pattern.query_map([], |row| {
            Ok(Playlist {
                id: row.get::<_, usize>(0).unwrap(),
                name: row.get::<_, String>(1).unwrap(),
                songs: None
            })
        }).unwrap().map(|x| x.unwrap()).collect::<Vec<Playlist>>();

        playlists.iter_mut().for_each(|playlist| self.load_playlist(playlist));
        playlists
    }

    pub fn get_playlist_by_id(&self, id: usize) -> Option<Playlist> {
        let mut pattern = self.connection.prepare("SELECT * FROM Playlist WHERE id = ?").unwrap();
        Some(pattern.query_map(params![id], |row| {
            let mut playlist = Playlist::new(id, row.get::<_, String>(1).unwrap(), None);
            self.load_playlist(&mut playlist);
            Ok(playlist)
        }).unwrap().map(|x| x.unwrap()).collect::<Vec<Playlist>>().remove(0))
    }

    pub fn add_song_to_cache(&self, song: &mut Song) {
        let _ = self.connection.execute("
            INSERT INTO Songs
            VALUES(null, ?1, ?2, ?3, ?4, ?5, ?6);
        ",
        params![song.id, song.name, song.artist, song.album, song.duration, if song.file == None { 0 } else { 1 }]);
        song.sql_id = self.connection.last_insert_rowid() as usize;
    }

    pub fn create_playlist(&self, name: String) -> Playlist {
        let _ = self.connection.execute("
            INSERT INTO Playlists
            VALUES(null, ?1);
        ",params![name]);
        println!("Created playlist. {} at ID {}", name, self.connection.last_insert_rowid());
        Playlist {
            id: self.connection.last_insert_rowid() as usize,
            name,
            songs: Some(vec![])
        }
    }

    pub fn add_song_to_playlist(&self, song: &Song, playlist: &mut Playlist) {
        let _ = self.connection.execute("
            INSERT INTO Contents
            VALUES(?1, ?2)
        ", params![playlist.id, song.sql_id]);
        match &mut playlist.songs {
            Some(songs) => songs.push(song.clone()),
            None => playlist.songs = Some(vec![song.clone()])
        }
    }
}
