use std::thread::sleep;
use std::io::{BufReader, BufRead};
use std::process::Command;
use thirtyfour_sync::prelude::*;
use std::time::Duration;
use std::process::Stdio;
use std::path::PathBuf;

use crate::application::Message;
use crate::music::Song;

pub async fn download(directory: PathBuf, mut target: Song) -> Message {
    let task_path = directory.join(PathBuf::from(format!("{}.mp3", target.id)));
    println!("[WORKER] Using task_path {}", task_path.to_string_lossy().to_string());
    if task_path.exists() {
        println!("[WORKER] {} is already downloaded", target.name);
        return Message::SuccessfulDownload(target);
    }

    let mut handle = Command::new("yt-dlp")
        .arg("-f")
        .arg("bestaudio")
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg("-o")
        .arg(format!("{}/{}.mp3", directory.to_string_lossy().to_string(), target.id))
        .arg(format!("https://music.youtube.com/watch?v={}", target.id))
        // .stdout(std::process::Stdio::null())
        //.stderr(std::process::Stdio::null())
        .spawn().unwrap();

    println!("[WORKER] Waiting for download {}", target.name);
    let _ = handle.wait();
    target.file = Some(task_path);
    println!("[WORKER] SuccessfulDownload({})", target.name);
    Message::SuccessfulDownload(target)
}

pub fn search_youtube_music(query: String, directory: PathBuf) -> Result<Vec<Song>, String> {
    let mut chromedriver = match Command::new("chromedriver").stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("Failed to spawn chromedriver: {e:?}"))
    };

    let stdout = match chromedriver.stdout.take() {
        Some(stdout) => stdout,
        None => return Err(String::from("Failed to capture STDOUT of chromedriver."))
    };

    let mut reader = BufReader::new(stdout).lines();

    for _ in 0..3 { let _ = reader.next(); }

    let ip = match reader.next() {
        Some(line) => line.unwrap().split(" ").nth(6).unwrap().to_string().strip_suffix('.').unwrap().to_string(),
        None => return Err(format!("Failed to read STDOUT of chromedriver."))
    };

    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_chrome_arg("--headless");
    let _ = caps.add_chrome_arg("--disable-gpu");
    let _ = caps.add_chrome_arg("--no-sandbox");
    let _ = caps.add_chrome_arg("--disable-software-rasterizer");
    let _ = caps.add_chrome_arg("--remote-debugging-port=9222");
    let _ = caps.add_chrome_arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

    let driver = WebDriver::new(format!("http://localhost:{ip}").as_str(), caps).unwrap();
    driver.get(format!("https://music.youtube.com/search?q={}", query.replace(" ", "+"))).unwrap();

    let songs_button = driver.find_elements(By::Css("a.yt-simple-endpoint")).unwrap();
    for element in songs_button {
        if element.is_clickable().unwrap() && element.text().unwrap() == "Songs" {
            element.click().unwrap();
            break;
        }
    }

    sleep(Duration::from_secs(1));

    let video_titles = driver.find_elements(By::ClassName("style-scope ytmusic-shelf-renderer")).unwrap();
    if video_titles.len() == 0 { return Ok(Vec::new()); }

    let songs = video_titles[0].clone();
    let urls = songs.find_elements(By::ClassName("yt-simple-endpoint")).unwrap();
    let mut url_list: Vec<String> = Vec::<String>::new();

    for url in urls {
        let addr = url.get_property("href").unwrap().unwrap();
        
        if match addr.chars().nth(26) {
            Some(c) => c == 'w',
            None => false
        } {
            url_list.push(addr.split("watch?v=").nth(1).unwrap().to_string())
        }
    }

    let mut lines = songs.text().unwrap().lines().skip(1).map(|x| x.to_string()).collect::<Vec<String>>();
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
        let timestr = lines.remove(0);
        let time = match timestr.contains(':') {
            true => timestr.split(':').map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>(),
            false => {
                vec![0, 0]
            }
        };
        let duration = time[0] * 60 + time[1];
        if duration != 0 { let _plays = lines.remove(0); }

        let id = url_list.remove(0);

        if duration == 0 {
            continue;
        }

        let path = directory.join(PathBuf::from(format!("{}.mp3", id)));

        let downloaded: Option<PathBuf> = match path.exists() {
            true => Some(path),
            false => None
        };

        options.push(Song::new(0, song, artist, album, id, duration, downloaded));
    }

    driver.quit().unwrap();
    Ok(options)
}
