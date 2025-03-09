mod application;
mod filemanager;
mod downloader;
mod utility;
mod widgets;
mod music;
mod audio;

use crate::application::Application;

fn main() -> iced::Result {
    iced::run("Resonate", Application::update, Application::view)
}
