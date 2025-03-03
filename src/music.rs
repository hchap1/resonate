use std::path::PathBuf;
use std::task::Waker;
use std::thread::JoinHandle;
use std::thread::spawn;

use iced::futures::Stream;

use crate::application::Message;
use crate::filemanager::Database;
use crate::downloader::search_youtube_music;
use crate::utility::*;

const DELIM: char = '͵'; // Unicode 0372, greek lower numeral sign

#[derive(Clone, PartialEq, Eq, Debug)]
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

// ------------ TASK ----------- //

pub struct SearchTask {
    query: String,
    database: AM<Database>,

    // Query songs into the queue and return them one by one
    queue: AMV<Song>,
    local_task: JoinHandle<()>,
    cloud_task: JoinHandle<()>,
    waker: AM<Option<Waker>>
}

impl SearchTask {
    pub fn new(query: String, database: AM<Database>) -> Self {
        let queue: AMV<Song> = sync(Vec::<Song>::new());

        let local_query = query.clone();
        let local_queue = queue.clone();
        let local_db = database.clone();
        let cloud_query = query.clone();
        let cloud_queue = queue.clone();
        let cloud_db = database.clone();

        println!("Task created: Search: {query}");

        let local_task = spawn(move || queue_local_search(local_query, local_db, local_queue));
        let cloud_task = spawn(move || queue_cloud_search(cloud_query, cloud_db, cloud_queue));

        Self {
            query,
            database,
            queue: sync(Vec::<Song>::new()),
            local_task,
            cloud_task,
            waker: sync(None)
        }
    }
}

fn queue_local_search(query: String, database: AM<Database>, dump: AMV<Song>) {
    println!("[LOCAL] Thread started.");
    let database = database.lock().unwrap();
    let mut dump = dump.lock().unwrap();
    dump.append(&mut database.search_cached_song(query));
    println!("[LOCAL] Thread finished.");
}

fn queue_cloud_search(query: String, database: AM<Database>, dump: AMV<Song>) {
    println!("[CLOUD] Thread started.");
    let (directory, db_hash) = {
        let database = database.lock().unwrap();
        (database.get_directory(), database.hash_all_songs())
    };

    let mut results: Vec<Song> = search_youtube_music(query, directory)
        .unwrap()
        .into_iter()
        .filter(|song| !db_hash.contains(&song.id))
        .collect();

    let database = database.lock().unwrap();
    database.add_songs_to_cache(&results);
    let mut dump = dump.lock().unwrap();
    dump.append(&mut results);
    dump.iter().for_each(|song| println!("[CLOUD] Song discovered: {}", song.display()));
    println!("[CLOUD] Thread finished.");
}

impl Stream for SearchTask {
    type Item = Message;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        
        // If the waker is not cached already, cache it
        'waker_cache: {
            {
                let waker = self.waker.lock().unwrap();
                if waker.is_some() { break 'waker_cache; }
            }
            self.waker = sync(Some(cx.waker().clone()));
        }

        println!("---------------------------- POLLED ---------------------------");
        let mut queue = self.queue.lock().unwrap();
        match queue.len() {
            0 => if self.cloud_task.is_finished() && self.local_task.is_finished() {
                println!("EXITING");
                std::task::Poll::Ready(None)
            } else {
                println!("PENDING");
                std::task::Poll::Pending
            },
            _ => {
                let song = queue.remove(0);
                println!("Found song: {}", song.display());
                std::task::Poll::Ready(Some(Message::IncomingSearch(song)))
            }
        }
    }
}
