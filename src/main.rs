mod downloader;
mod filemanager;
mod music;

use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use downloader::Task;

use crate::music::Song;
use crate::downloader::Downloader;
use crate::filemanager::Database;
use crate::filemanager::get_application_directory;

fn main() {
    let directory: PathBuf = get_application_directory().unwrap();
    let database: Database = Database::new(directory);
    database.add_song_to_cache(&Song::new(
        String::from("Paradise"),
        String::from("Coldplay"),
        String::from("some_id"),
        0,
        None
    ));

    let res = database.search_cached_song(String::from("para"));
    for r in res {
        println!("SONG -> {}", r.name);
    }
}
