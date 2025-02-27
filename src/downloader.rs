use std::thread::{JoinHandle, spawn, sleep};
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::Command;
use std::sync::{Arc, Mutex};
use thirtyfour::prelude::*;
use std::time::Duration;
use std::process::Stdio;
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

                let task_path = directory.join(PathBuf::from(format!("{}.mp3", self.target.id)));
                println!("[WORKER] Using task_path {}", task_path.to_string_lossy().to_string());
                if task_path.exists() {
                    return;
                }

                let mut handle = Command::new("yt-dlp")
                    .arg("-f")
                    .arg("bestaudio")
                    .arg("--extract-audio")
                    .arg("--audio-format")
                    .arg("mp3")
                    .arg("-o")
                    .arg(format!("{}/{}.mp3", directory.to_string_lossy().to_string(), self.target.id))
                    .arg(format!("https://music.youtube.com/watch?v={}", self.target.id))
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

pub async fn search_youtube_music(query: String, directory: PathBuf) -> Result<Vec<Song>, String> {
    let mut chromedriver = match Command::new("chromedriver").stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("Failed to spawn chromedriver: {e:?}"))
    };

    let stdout = match chromedriver.stdout.take() {
        Some(stdout) => stdout,
        None => return Err(String::from("Failed to capture STDOUT of chromedriver."))
    };

    let mut reader = BufReader::new(stdout).lines();

    for _ in 0..3 { let _ = reader.next_line().await; }

    let ip = match reader.next_line().await {
        Ok(line) => line.unwrap().split(" ").nth(6).unwrap().to_string().strip_suffix('.').unwrap().to_string(),
        Err(e) => return Err(format!("Failed to read STDOUT of chromedriver: {e:?}"))
    };

    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--headless");
    let _ = caps.add_arg("--disable-gpu");
    let _ = caps.add_arg("--no-sandbox");
    let _ = caps.add_arg("--disable-software-rasterizer");
    let _ = caps.add_arg("--remote-debugging-port=9222");
    let _ = caps.add_arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

    let driver = WebDriver::new(format!("http://localhost:{ip}"), caps).await.unwrap();
    driver.goto(format!("https://music.youtube.com/search?q={}", query)).await.unwrap();

    let button = driver.find_all(By::Css("button.yt-spec-button-shape-next")).await.unwrap();

    for element in button {
        if element.text().await.unwrap() == "Show all" && element.is_clickable().await.unwrap() {
            element.click().await.unwrap();
            break;
        }
    }

    sleep(Duration::from_secs(1));

    let video_titles = driver.find_all(By::ClassName("style-scope ytmusic-shelf-renderer")).await.unwrap();
    if video_titles.len() == 0 { return Ok(Vec::new()); }

    let songs = video_titles[0].clone();
    let urls = songs.find_all(By::ClassName("yt-simple-endpoint")).await.unwrap();
    let mut url_list: Vec<String> = Vec::<String>::new();

    for url in urls {
        let addr = url.prop("href").await.unwrap().unwrap();
        
        if match addr.chars().nth(26) {
            Some(c) => c == 'w',
            None => false
        } {
            url_list.push(addr.split("watch?v=").nth(1).unwrap().to_string())
        }
    }

    let mut lines = songs.text().await.unwrap().lines().skip(1).map(|x| x.to_string()).collect::<Vec<String>>();
    let mut options: Vec<Song> = Vec::<Song>::new();

    while lines.len() >= 7 {
        let song = lines.remove(0);
        let mut artist = lines.remove(0);
        loop {
            let item = lines.remove(0);
            if item != " • " {
                artist += item.as_str();
            } else {
                break;
            }
        }
        let album = lines.remove(0);
        lines.remove(0);
        let time = lines.remove(0).split(':').map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();
        let duration = time[0] * 60 + time[1];
        let _plays = lines.remove(0);

        let id = url_list.remove(0);

        let path = directory.join(PathBuf::from(format!("{}.mp3", id)));
        let downloaded: Option<PathBuf> = match path.exists() {
            true => Some(path),
            false => None
        };

        options.push(Song::new(song, artist, id, album, duration, downloaded));
    }

    driver.quit().await.unwrap();
    Ok(options)
}
