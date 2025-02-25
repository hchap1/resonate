use std::thread::{JoinHandle, spawn, sleep};
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::time::Duration;
use std::path::PathBuf;

use crate::music::Song;

type AM<T> = Arc<Mutex<T>>;
type AMV<T> = Arc<Mutex<Vec<T>>>;

fn sync<T>(obj: T) -> AM<T> {
    Arc::new(Mutex::new(obj))
}

enum Action {
    Download,
    Remove
}

struct Task {
    target: Song,
    action: Action
}

struct Downloader {
    tasks: Vec<Task>,
    workers: Vec<Worker>,
    directory: PathBuf
}

struct Worker {
    running: AM<bool>,
    todo: AMV<Task>,
    handle: Option<JoinHandle<()>>,
    directory: PathBuf
}

/// Spawn a worker thread to complete a task
fn worker_thread(todo_ref: AMV<Task>, running_ref: AM<bool>, directory: PathBuf) {
    let period: Duration = Duration::from_secs(1);
    loop {
        // Check whether to exit then implicitly release mutex
        {
            let running = running_ref.lock().unwrap();
            if !*running {
                return;
            }
        } {
            let mut todo = todo_ref.lock().unwrap();
            if todo.len() == 0 {

                // If there is nothing left to do, terminate the thread.
                let mut running = running_ref.lock().unwrap();
                *running = false;

            } else {

                // Else, complete the task, blocking until it finishes then removing it
                todo.remove(0).execute(&directory);
            }
        }
        sleep(period);
    }
}

impl Worker {
    pub fn new(directory: PathBuf) -> Self {
        let todo: AMV<Task> = sync::<Vec<Task>>(Vec::<Task>::new());
        let running: AM<bool> = sync::<bool>(true);
        Self { running, todo, handle: None, directory }
    }

    fn queue(&mut self, task: Task) {
        let mut running = self.running.lock().unwrap();

        // If the thread is already running, simply add this to the queue
        // Else, make a thread and do the same
        if running {
            
        }
    }
}

impl Task {
    pub fn download(song: Song) -> Self {
        Self { target: song, action: Action::Download }
    }

    fn execute(&mut self, directory: &PathBuf) {
        match self.action {

            Action::Download => {
                // Create a command that runs an instance of yt-dlp

                let handle = Command::new("yt-dlp").arg("-f").arg("bestaudio").arg("--extract-audio").arg("--audio-format").arg("mp3").arg("-o").arg(format!(
                    "{}/{}.mp3", directory.to_string_lossy().to_string(), file_id
                )).arg(&self.url).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            }

            Action::Remove => {

            }
        }
    }
}

impl Downloader {
    pub fn new(directory: PathBuf) -> Self {
        let mut workers: Vec<Worker> = Vec::new();
        for _ in 0..4 { workers.push(Worker::new(directory.clone())); }
        Self { tasks: Vec::new(), workers, directory }
    }

    pub fn async_execute(&mut self, task: Task) {
        
    }
}
