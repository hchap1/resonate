use std::thread::{JoinHandle, spawn, sleep};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    workers: Vec<JoinHandle<()>>
}

struct Worker {
    running: AM<bool>,
    todo: AMV<Task>,
    handle: JoinHandle<()>
}

/// Spawn a worker thread, querying for tasks constantly
fn worker_thread(todo_ref: AMV<Task>, running_ref: AM<bool>) {
    let period: Duration = Duration::from_secs(1);
    loop {
        sleep(period);
        // Check whether to exit then implicitly release mutex
        {
            let running = running_ref.lock().unwrap();
            if !*running {
                return;
            }
        }
    }
}

impl Worker {
    pub fn new() -> Self {
        let todo: AMV<Task> = sync::<Vec<Task>>(Vec::<Task>::new());
        let running: AM<bool> = sync::<bool>(true);
        let todo_ref = Arc::clone(&todo);
        let running_ref = Arc::clone(&running);
        let handle: JoinHandle<()> = spawn(move || worker_thread(todo_ref, running_ref));
    }
}

impl Task {
    pub fn download(song: Song) -> Self {
        Self { target: song, action: Action::Download }
    }
}

impl Downloader {
    pub fn new() -> Self {
        Self { tasks: Vec::new(), workers: Vec::new() }
    }

    pub fn async_execute(&mut self, task: Task) {
        
    }
}
