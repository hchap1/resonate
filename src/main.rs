mod downloader;
mod filemanager;
mod music;

use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use downloader::Task;

use crate::music::Song;
use crate::downloader::Downloader;
use crate::filemanager::get_application_directory;

fn main() {
    let directory: PathBuf = get_application_directory().unwrap();
    println!("directory: {}", directory.to_string_lossy().to_string());
    let mut downloader: Downloader = Downloader::new(directory);


    let song1: Song = Song::new(
        String::from("Clocks"),
        String::from("Coldplay"),
        String::from("8Xv_Hg8o1fw"),
        None
    );

    let song2: Song = Song::new(
        String::from("Paradise"),
        String::from("Coldplay"),
        String::from("Q0TEUMPIhk8"),
        None
    );

    downloader.async_execute(Task::download(song1));
    downloader.async_execute(Task::download(song2));

    loop {
        sleep(Duration::from_millis(300));
        let workers = downloader.get_busy_worker_count();
        println!("BUSY WORKERS: {workers}");
        if workers == 0 { break; }
    }
}
