use std::thread::{self, JoinHandle};

use flume::{Receiver, Sender};

use crate::audio::{AudioCommand, PlaybackSnapshot, SpectrogramBins};

pub struct AudioManager {
    cmd_tx: Sender<AudioCommand>,
    playback_thread_handle: Option<JoinHandle<()>>,
    progress_rx: Receiver<PlaybackSnapshot>,
    spectrogram_rx: Receiver<SpectrogramBins>,
    audio_file_path: Option<String>,
}

impl AudioManager {
    pub fn new(audio_file_path: Option<String>) -> Self {
        let (cmd_tx, _) = flume::unbounded::<AudioCommand>();
        let (_, progress_rx) = flume::bounded::<PlaybackSnapshot>(8);
        let (_, spectrogram_rx) = flume::bounded::<SpectrogramBins>(32);

        Self {
            cmd_tx,
            playback_thread_handle: None,
            progress_rx,
            spectrogram_rx,
            audio_file_path,
        }
    }

    pub fn spawn_playback_thread(&mut self) {
        if self.playback_thread_handle.is_some() {
            return;
        }

        let audio_file_path = self.audio_file_path.clone();
        let (cmd_tx, cmd_rx) = flume::unbounded::<AudioCommand>();
        let (progress_tx, progress_rx) = flume::bounded::<PlaybackSnapshot>(8);
        let (spectrogram_tx, spectrogram_rx) = flume::bounded::<SpectrogramBins>(32);

        self.cmd_tx = cmd_tx;
        self.progress_rx = progress_rx;
        self.spectrogram_rx = spectrogram_rx;

        let handle = thread::Builder::new()
            .name("audio-thread".into())
            .spawn(move || {
                run_playback_thread(cmd_rx, progress_tx, spectrogram_tx, audio_file_path)
            })
            .expect("failed to spawn audio thread");

        self.playback_thread_handle = Some(handle);
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

    pub fn seek(&self, seconds: f64) {
        let _ = self.cmd_tx.send(AudioCommand::Seek(seconds));
    }

    pub fn progress(&self) -> &Receiver<PlaybackSnapshot> {
        &self.progress_rx
    }

    pub fn spectrogram(&self) -> &Receiver<SpectrogramBins> {
        &self.spectrogram_rx
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Terminate);

        if let Some(handle) = self.playback_thread_handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_playback_thread(
    rx: Receiver<AudioCommand>,
    progress_tx: Sender<PlaybackSnapshot>,
    spectrogram_tx: Sender<SpectrogramBins>,
    initial_track: Option<String>,
) {
    let mut loop_state = match super::audio_worker::AudioWorker::new(progress_tx, spectrogram_tx) {
        Ok(loop_state) => loop_state,
        Err(err) => {
            log::warn!("audio: failed to init playback loop: {err}");
            return;
        }
    };

    if let Some(path) = initial_track {
        loop_state.handle_command(AudioCommand::LoadFile(path));
    }

    loop {
        match rx.recv() {
            Ok(cmd) => {
                let should_exit = matches!(cmd, AudioCommand::Terminate);
                loop_state.handle_command(cmd);

                if should_exit {
                    break;
                }
            }
            Err(_) => {
                log::warn!("audio palyback thread: channel closed");
                break;
            }
        }
    }
}
