use std::fs::File;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use flume::{Receiver, Sender};
use rodio::{Decoder, OutputStreamBuilder, Sink};

#[derive(Debug)]
pub enum AudioCommand {
    LoadFile(String),
    Play,
    Pause,
    Stop,
    Terminate,
}

pub struct AudioRuntime {
    cmd_tx: Sender<AudioCommand>,
    handle: Option<JoinHandle<()>>,
}

pub fn is_audio_path(path: &str) -> bool {
    matches!(file_ext(path), Some(ext) if ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("flac"))
}

fn file_ext(path: &str) -> Option<&str> {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
}

impl AudioRuntime {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = flume::unbounded::<AudioCommand>();

        let handle = thread::Builder::new()
            .name("audio-thread".into())
            .spawn(move || audio_worker(cmd_rx))
            .expect("failed to spawn audio thread");

        Self {
            cmd_tx,
            handle: Some(handle),
        }
    }

    pub fn load_file(&self, path: String) {
        let _ = self.cmd_tx.send(AudioCommand::LoadFile(path));
    }

    pub fn play(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Pause);
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Stop);
    }
}

impl Drop for AudioRuntime {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Terminate);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn audio_worker(rx: Receiver<AudioCommand>) {
    let stream = match OutputStreamBuilder::open_default_stream() {
        Ok(stream) => stream,
        Err(err) => {
            log::warn!("audio: no output device available; audio disabled: {err}");
            return;
        }
    };

    let mixer = stream.mixer();
    let mut sink: Option<Sink> = None;
    let mut current_track: Option<String> = None;

    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCommand::LoadFile(path) => {
                log::info!("audio: load file {path}");
                current_track = Some(path.clone());
                if let Some(s) = sink.take() {
                    s.stop();
                }
                sink = load_sink(mixer, &path).ok();
            }
            AudioCommand::Play => {
                log::info!("audio: play");
                if let Some(s) = sink.as_ref() {
                    s.play();
                }
            }
            AudioCommand::Pause => {
                log::info!("audio: pause");
                if let Some(s) = sink.as_ref() {
                    s.pause();
                }
            }
            AudioCommand::Stop => {
                log::info!("audio: stop");
                if let Some(track) = current_track.clone() {
                    sink = seek_or_load_on_position(Duration::ZERO, sink, track, mixer);
                }
            }
            AudioCommand::Terminate => {
                log::info!("audio: terminate");
                if let Some(s) = sink.take() {
                    s.stop();
                }
                break;
            }
        }
    }
}

fn seek_or_load_on_position(
    pos: Duration,
    sink: Option<Sink>,
    track: String,
    mixer: &rodio::mixer::Mixer,
) -> Option<Sink> {
    if seek(pos, sink.as_ref()).is_ok() {
        return sink;
    }
    load_sink_on_position(pos, track, mixer)
}

fn seek(pos: Duration, sink: Option<&Sink>) -> Result<(), ()> {
    let s = sink.ok_or(())?;
    s.try_seek(pos).map_err(|e| {
        log::warn!("audio: failed to seek: {e}");
    })?;
    s.pause();
    Ok(())
}

fn load_sink_on_position(
    pos: Duration,
    track: String,
    mixer: &rodio::mixer::Mixer,
) -> Option<Sink> {
    let new_sink = load_sink(mixer, &track).ok()?;
    if pos > Duration::ZERO {
        let _ = new_sink.try_seek(pos);
    }
    Some(new_sink)
}

fn load_sink(mixer: &rodio::mixer::Mixer, path: &str) -> Result<Sink, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let source = Decoder::try_from(file).map_err(|e| e.to_string())?;
    let sink = Sink::connect_new(mixer);
    sink.pause();
    sink.append(source);
    Ok(sink)
}
