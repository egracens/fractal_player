use std::thread::{self, JoinHandle};
use std::time::Duration;

use flume::{Receiver, Sender};

#[derive(Debug)]
pub enum AudioCommand {
    LoadFile(String),
    Play,
    Pause,
    Stop,
}

pub struct AudioRuntime {
    cmd_tx: Sender<AudioCommand>,
    handle: Option<JoinHandle<()>>,
}

pub fn is_mp3_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("mp3"))
        .unwrap_or(false)
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
        let _ = self.cmd_tx.send(AudioCommand::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn audio_worker(rx: Receiver<AudioCommand>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCommand::LoadFile(path) => log::info!("audio: load file {path}"),
            AudioCommand::Play => log::info!("audio: play"),
            AudioCommand::Pause => log::info!("audio: pause"),
            AudioCommand::Stop => {
                log::info!("audio: stop");
                break;
            }
        }

        // Placeholder work to illustrate threading without real audio I/O.
        thread::sleep(Duration::from_millis(5));
    }
}
