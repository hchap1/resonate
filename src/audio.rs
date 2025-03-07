use std::fs::File;
use std::io::BufReader;
use std::collections::VecDeque;
use std::thread::JoinHandle;
use std::time::Duration;
use std::thread::sleep;
use std::thread::spawn;

use rodio::Sink;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamHandle;

use crate::utility::*;
use crate::music::Song;

pub struct AudioPlayer {
    
    // Required to keep audio in scope
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    _queue_handle: JoinHandle<()>,

    sink: AM<Sink>,
    queue: AMQ<Song>,
    current: AMO<Song>
}

pub fn queueing_thread(sink: AM<Sink>, queue: AMQ<Song>, current: AMO<Song>) {
    let sleep_duration = Duration::from_secs(1);
    loop {
        sleep(sleep_duration);

        let sink = sink.lock().unwrap();

        // If we need to queue the next song
        if sink.empty() {
            let mut queue = queue.lock().unwrap();
            let mut current = current.lock().unwrap();
            *current = queue.pop_front();

            let song = match current.as_ref() {
                Some(song_ref) => song_ref.clone(),
                None => continue
            };

            let file = BufReader::new(File::open(song.file.as_ref().unwrap()).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
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

        let sink = sync(sink);
        let queue = sync(VecDeque::new());
        let current = sync(None);

        let sink_clone = sink.clone();
        let queue_clone = queue.clone();
        let current_clone = current.clone();

        let _queue_handle = spawn(move || queueing_thread(sink_clone, queue_clone, current_clone));

        Ok(Self {
            _stream: stream,
            _handle: handle,
            sink,
            queue,
            current,
            _queue_handle
        })
    }


    pub fn queue_song(&mut self, song: Song) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(song);
    }

    pub fn play(&mut self, song: Song) {
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
    }

    pub fn skip_song(&mut self) {
        let sink = self.sink.lock().unwrap();
        println!("[AUDIO] Song skipped");
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

    pub fn append(&mut self, songs: Vec<Song>) {
        let mut queue = self.queue.lock().unwrap();
        songs.into_iter().for_each(|song| queue.push_back(song));
    }

    pub fn clear(&mut self) {
        let mut queue = self.queue.lock().unwrap();
        let sink = self.sink.lock().unwrap();
        queue.clear();
        sink.clear();
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
}
