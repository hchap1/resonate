use std::fs::File;
use std::io::BufReader;
use std::collections::VecDeque;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::Duration;
use std::thread::sleep;
use std::thread::spawn;

use rodio::Sink;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Source;

use crate::application::Message;
use crate::utility::*;
use crate::music::Song;

pub struct AudioPlayer {
    
    // Required to keep audio in scope
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    _queue_handle: JoinHandle<()>,

    sink: AM<Sink>,
    queue: AMQ<Song>,
    current: AMO<Song>,
    progress: AM<f32>,
    looping_song: AM<bool>
}

pub fn queueing_thread(sink: AM<Sink>, queue: AMQ<Song>, current: AMO<Song>, progress: AM<f32>, looping_song: AM<bool>) {
    let sleep_duration = Duration::from_secs(1);
    loop {
        sleep(sleep_duration);

        let sink = sink.lock().unwrap();
        let looping_song = looping_song.lock().unwrap();

        // If we need to queue the next song
        if sink.empty() {
            let mut progress = progress.lock().unwrap();
            *progress = 0f32;
            let mut queue = queue.lock().unwrap();
            let mut current = current.lock().unwrap();
            if !*looping_song { *current = queue.pop_front(); }

            let song = match current.as_ref() {
                Some(song_ref) => song_ref.clone(),
                None => continue
            };

            let file = BufReader::new(File::open(song.file.as_ref().unwrap()).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
        } else if !sink.is_paused() {
            let mut progress = progress.lock().unwrap();
            *progress += (sleep_duration.as_secs() as f32) * sink.speed();
        }
    }
}

impl AudioPlayer {
    pub fn new() -> Result<Self, ()> {
        let (stream, handle) = match OutputStream::try_default() {
            Ok(data) => data,
            Err(_) => return Err(())
        };

        let sink = match Sink::try_new(&handle) {
            Ok(sink) => sink,
            Err(_) => return Err(())
        };

        sink.set_volume(0.2);

        let sink = sync(sink);
        let queue = sync(VecDeque::new());
        let current = sync(None);
        let progress = sync(0f32);
        let looping_song = sync(false);

        let sink_clone = sink.clone();
        let queue_clone = queue.clone();
        let current_clone = current.clone();
        let progress_clone = progress.clone();
        let looping_song_clone = looping_song.clone();

        let _queue_handle = spawn(move || queueing_thread(sink_clone, queue_clone, current_clone, progress_clone, looping_song_clone));

        Ok(Self {
            _stream: stream,
            _handle: handle,
            sink,
            queue,
            current,
            _queue_handle,
            progress,
            looping_song
        })
    }


    pub fn queue_song(&mut self, song: Song) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(song);
        let mut current = self.current.lock().unwrap();
        if current.is_none() {
            *current = Some(queue[0].clone());
        }
    }

    pub fn play(&mut self, song: Song) {
        self.resume();
        println!("[AUDIO] Received play command for {}", song.name);
        self.insert_song(song.clone());
        println!("[AUDIO] Added song to queue");
        self.skip_song();
        println!("[AUDIO] Skipped current song");
        let mut current = self.current.lock().unwrap();
        *current = Some(song);
        println!("[AUDIO] Updated current");
    }

    pub fn insert_song(&mut self, song: Song) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_front(song);
        let mut current = self.current.lock().unwrap();
        *current = Some(queue[0].clone());
    }

    pub fn skip_song(&mut self) {
        let sink = self.sink.lock().unwrap();
        println!("[AUDIO] Song skipped");
        let queue = self.queue.lock().unwrap();
        if queue.len() > 0 {
            let mut current = self.current.lock().unwrap();
            *current = Some(queue[0].clone());
        } else {
            let mut current = self.current.lock().unwrap();
            *current = None;
        }
        sink.skip_one();
    }

    pub fn pause(&self) {
        let sink = self.sink.lock().unwrap();
        sink.pause();
    }

    pub fn resume(&self) {
        let sink = self.sink.lock().unwrap();
        sink.play();
    }

    pub fn is_paused(&self) -> bool {
        let sink = self.sink.lock().unwrap();
        sink.is_paused()
    }

    pub fn get_current(&self) -> Option<Song> {
        let current = self.current.lock().unwrap();
        match current.as_ref() {
            Some(song_ref) => Some(song_ref.clone()),
            None => None
        }
    }

    pub fn is_this_playing(&self, song: &Song) -> bool {
        let current = self.current.lock().unwrap();
        match current.as_ref() {
            Some(song_ref) => song_ref.sql_id == song.sql_id,
            None => false
        }
    }

    pub fn get_queue(&self) -> Vec<Song> {
        let queue = self.queue.lock().unwrap();
        let r: Vec<Song> = queue.clone().into();
        let remove_first = if let Some(c) = self.get_current() {
            if r.len() > 0 {
                if r[0].sql_id == c.sql_id {
                    true
                } else { false }
            } else { false }
        } else { false };
        if remove_first { r[1..].to_vec() } else { r }
    }

    pub fn slow(&self) {
        let sink = self.sink.lock().unwrap();
        sink.set_speed(0.85);
    }

    pub fn fast(&self) {
        let sink = self.sink.lock().unwrap();
        sink.set_speed(1.4);
    }
    
    pub fn normal(&self) {
        let sink = self.sink.lock().unwrap();
        sink.set_speed(1f32)
    }
    
    pub fn get_progress_source(&self) -> AM<f32> { self.progress.clone() }

    pub fn get_volume(&self) -> f32 {
        let sink = self.sink.lock().unwrap();
        sink.volume() * 200f32
    }

    pub fn set_volume(&self, volume: f32) {
        let sink = self.sink.lock().unwrap();
        sink.set_volume(volume / 200f32)
    }

    pub fn is_looping(&self) -> bool {
        let looping = self.looping_song.lock().unwrap();
        *looping
    }

    pub fn set_looping(&mut self, do_looping: bool) {
        let mut looping = self.looping_song.lock().unwrap();
        *looping = do_looping;
    }
}

pub async fn get_progress(progress_source: AM<f32>) -> Message {
    let progress = progress_source.lock().unwrap();
    sleep(Duration::from_secs(1));
    Message::ProgressUpdate(*progress)
}

pub fn query_song_length(path: &PathBuf) -> usize {
    let file = File::open(path).expect("Failed to open file");
    println!("Read file, length: {}", file.metadata().unwrap().size());
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader).expect("Failed to decode file");
    let sample_rate = decoder.sample_rate() as f64;
    let num_samples = decoder.total_duration().unwrap_or_default().as_secs_f64() * sample_rate;
    (num_samples / sample_rate).round() as usize
}
