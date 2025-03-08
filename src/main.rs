mod application;
mod filemanager;
mod downloader;
mod utility;
mod widgets;
mod music;
mod audio;

use std::path::PathBuf;

use music::Song;

use crate::application::Application;
use crate::downloader::convert_and_save_song;

/*
fn main() -> iced::Result {
    iced::run("Resonate", Application::update, Application::view)
}
*/

fn main() {
    let mut song: Song = Song::new(0,
        String::from("TestSong"),
        String::from("TestArtist"),
        String::from("TestAlbum"),
        String::from("TestID"),
        0,
        Some(PathBuf::from("C:/users/hchap/Downloads/pepper_green_eyes.m4a"))
    );

    convert_and_save_song(PathBuf::from("C:/users/hchap/Downloads"), &mut song);
}
