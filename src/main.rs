mod application;
mod filemanager;
mod downloader;
mod utility;
mod widgets;
mod music;

use std::path::PathBuf;

use crate::downloader::Downloader;
use crate::filemanager::Database;
use crate::filemanager::get_application_directory;
use crate::application::Application;

fn main() -> iced::Result {
    let directory: PathBuf = get_application_directory().unwrap();
    let database: Database = Database::new(directory.clone());
    let _downloader: Downloader = Downloader::new(directory.clone());

    println!("DATABASE CONTENTS:");
    database.retrieve_all_songs().iter().for_each(|song| println!("{}", song.display()));

    iced::run("Resonate", Application::update, Application::view)
}
