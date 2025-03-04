mod application;
mod filemanager;
mod downloader;
mod utility;
mod widgets;
mod music;

use crate::application::Application;

fn main() -> iced::Result {
    iced::run("Resonate", Application::update, Application::view)
}
