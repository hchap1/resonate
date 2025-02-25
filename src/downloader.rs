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

pub struct Task {
    target: Song,
    action: Action
}

pub struct Downloader {
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
        }

        let task: Option<Task> = {
            let mut todo = todo_ref.lock().unwrap();
            if todo.len() == 0 {

                // If there is nothing left to do, terminate the thread.
                let mut running = running_ref.lock().unwrap();
                *running = false;
                None

            } else {

                // Else, complete the task, blocking until it finishes then removing it
                Some(todo.remove(0))
            }
        };

        if let Some(mut task) = task {
            task.execute(&directory);
        }

        sleep(period);
    }
}

impl Worker {
    pub fn new(directory: PathBuf) -> Self {
        let todo: AMV<Task> = sync::<Vec<Task>>(Vec::<Task>::new());
        let running: AM<bool> = sync::<bool>(false);
        Self { running, todo, handle: None, directory }
    }

    fn queue(&mut self, task: Task) {
        let running = { if *self.running.lock().unwrap() { true } else { false } };

        // If the thread is already running, simply add this to the queue
        // Else, make a thread and do the same
        if running {

            let mut todo = self.todo.lock().unwrap();
            todo.push(task);

        } else {

            let todo_ref = Arc::clone(&self.todo);
            let running_ref = Arc::clone(&self.running);
            let directory_clone = self.directory.clone();

            {
                let mut todo = self.todo.lock().unwrap();
                todo.push(task);

                let mut running = self.running.lock().unwrap();
                *running = true;
            }

            self.handle = Some(spawn(move || worker_thread(todo_ref, running_ref, directory_clone)));
        }
    }

    fn query_task_length(&self) -> Option<usize> {
        let running = self.running.lock().unwrap();
        let todo = self.todo.lock().unwrap();
        if *running { Some(todo.len()) } else { None }
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

                let mut handle = Command::new("yt-dlp")
                    .arg("-f")
                    .arg("bestaudio")
                    .arg("--extract-audio")
                    .arg("--audio-format")
                    .arg("mp3")
                    .arg("-o")
                    .arg(format!("{}/{}.mp3", directory.to_string_lossy().to_string(), self.target.id))
                    .arg(format!("https://music.youtube.com/watch?v={}", &self.target.id))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn().unwrap();

                let _ = handle.wait();

                println!("[WORKER] finished downloading {}", self.target.name);

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
        
        let mut minimum_workload: usize = 0;
        let mut most_free_worker: usize = 0;
        
        for idx in 0..self.workers.len() {
            match self.workers[idx].query_task_length() {
                Some(number_of_tasks) => {
                    if number_of_tasks < minimum_workload {
                        most_free_worker = idx;
                        minimum_workload = number_of_tasks;
                    }
                }
                None => {
                    self.workers[idx].queue(task);
                    return;
                }
            }
        }

        self.workers[most_free_worker].queue(task);

    }

    pub fn get_busy_worker_count(&self) -> usize {
        self.workers.iter().map(|worker| {
            if *worker.running.lock().unwrap() { 1 } else { 0 }
        }).sum::<usize>()
    }
}
