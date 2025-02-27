mod downloader;
mod filemanager;
mod utility;
mod music;

use std::path::PathBuf;

use crate::downloader::Downloader;
use crate::filemanager::Database;
use crate::filemanager::get_application_directory;

#[tokio::main]
async fn main() {
    let directory: PathBuf = get_application_directory().unwrap();
    let database: Database = Database::new(directory.clone());
    let downloader: Downloader = Downloader::new(directory);
}
