mod application;
mod filemanager;
mod downloader;
mod utility;
mod widgets;
mod music;
mod audio;

use application::Message;
use iced::Task;

use crate::application::Application;

fn main() -> iced::Result {
    iced::application("Resonate", Application::update, Application::view)
        .subscription(|_| Application::keyboard_subscription())
        .run_with(|| (Application::default(), Task::done(Message::CheckAllIcons)))
}
