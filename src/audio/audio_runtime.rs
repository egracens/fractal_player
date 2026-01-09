use std::thread::{self, JoinHandle};

use flume::{Receiver, Sender};

use crate::audio::{AudioCommand, PlaybackSnapshot, SpectrogramSlice};

pub struct AudioRuntime {
    cmd_tx: Sender<AudioCommand>,
    playback_thread_handle: Option<JoinHandle<()>>,
    progress_rx: Receiver<PlaybackSnapshot>,
    spectrogram_rx: Receiver<SpectrogramSlice>,
    audio_file_path: Option<String>,
}

impl AudioRuntime {
    pub fn new(audio_file_path: Option<String>) -> Self {
        Self {
            cmd_tx: flume::unbounded::<AudioCommand>().0,
            playback_thread_handle: None,
            progress_rx: flume::bounded::<PlaybackSnapshot>(8).1,
            spectrogram_rx: flume::bounded::<SpectrogramSlice>(32).1,
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
        let (spectrogram_tx, spectrogram_rx) = flume::bounded::<SpectrogramSlice>(32);

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

    pub fn progress(&self) -> &Receiver<PlaybackSnapshot> {
        &self.progress_rx
    }

    pub fn spectrogram(&self) -> &Receiver<SpectrogramSlice> {
        &self.spectrogram_rx
    }
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Drop for AudioRuntime {
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
    spectrogram_tx: Sender<SpectrogramSlice>,
    initial_track: Option<String>,
) {
    let mut loop_state = match super::playback_loop::PlaybackLoop::new(progress_tx, spectrogram_tx)
    {
        Ok(loop_state) => loop_state,
        Err(err) => {
            log::warn!("audio: failed to init playback loop: {err}");
            return;
        }
    };

    if let Some(path) = initial_track {
        loop_state.handle_command(AudioCommand::LoadFile(path));
    }

    let timeout = std::time::Duration::from_millis(200);

    loop {
        match rx.recv_timeout(timeout) {
            Ok(cmd) => {
                let should_exit = matches!(cmd, AudioCommand::Terminate);
                loop_state.handle_command(cmd);
                if should_exit {
                    break;
                }
            }
            Err(flume::RecvTimeoutError::Timeout) => loop_state.tick(),
            Err(flume::RecvTimeoutError::Disconnected) => break,
        }
    }
}
