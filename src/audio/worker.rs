use std::time::Duration;

use flume::{Receiver, Sender};

use crate::audio::{AudioCommand, PlaybackSnapshot};

use super::playback_loop::PlaybackLoop;

pub fn run_worker(
    rx: Receiver<AudioCommand>,
    progress_tx: Sender<PlaybackSnapshot>,
    initial_track: Option<String>,
) {
    let mut loop_state = match PlaybackLoop::new(progress_tx) {
        Ok(loop_state) => loop_state,
        Err(err) => {
            log::warn!("audio: failed to init playback loop: {err}");
            return;
        }
    };

    if let Some(path) = initial_track {
        loop_state.handle_command(AudioCommand::LoadFile(path));
    }

    let timeout = Duration::from_millis(200);

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
