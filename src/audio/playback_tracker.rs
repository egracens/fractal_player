use flume::Sender;

use crate::audio::PlaybackSnapshot;

const PROGRESS_STRIDE_FRAMES: u32 = 1024;

pub struct PlaybackTracker {
    sample_rate: u32,
    frames: u64,
    duration_secs: Option<f64>,
    tx: Sender<PlaybackSnapshot>,
}

impl PlaybackTracker {
    pub fn new(sample_rate: u32, duration_secs: Option<f64>, tx: Sender<PlaybackSnapshot>) -> Self {
        Self {
            sample_rate,
            frames: 0,
            duration_secs,
            tx,
        }
    }

    pub fn on_frame(&mut self, is_playing: bool) {
        self.frames = self.frames.saturating_add(1);
        if (self.frames as u32) % PROGRESS_STRIDE_FRAMES == 0 {
            self.send_snapshot(is_playing);
        }
    }

    pub fn send_snapshot(&self, is_playing: bool) {
        let pos_secs = self.frames as f64 / self.sample_rate as f64;
        let snapshot = PlaybackSnapshot {
            pos_secs,
            duration_secs: self.duration_secs.unwrap_or(0.0),
            is_playing,
        };
        let _ = self.tx.send(snapshot);
    }
}
