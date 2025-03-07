use std::fs::File;
use std::io::BufReader;
use std::collections::VecDeque;
use std::time::Duration;
use std::thread::sleep;

use rodio::Sink;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamHandle;

use crate::application::Message;
use crate::utility::*;
use crate::music::Song;

pub struct AudioPlayer {
    
    // Required to keep audio in scope
    _stream: OutputStream,
    _handle: OutputStreamHandle,

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

        Ok(Self {
            _stream: stream,
            _handle: handle,
            sink: sync(sink),
            queue: sync(VecDeque::new()),
            current: sync(None)
        })
    }

    pub fn queue_song(&mut self, song: Song) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(song);
    }

    pub fn insert_song(&mut self, song: Song) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_front(song);
    }

    pub fn skip_song(&mut self) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front();
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
}
