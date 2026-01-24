use flume::Sender;

use crate::audio::{PlaybackSnapshot, SampleConsumer};

const PROGRESS_STRIDE_SECONDS: f64 = 0.1;

pub struct PlaybackTracker {
    sample_rate: u32,
    frames_count_tracked: u64,
    frames_per_stride: u64,
    duration_secs: Option<f64>,
    tx: Sender<PlaybackSnapshot>,
}

impl PlaybackTracker {
    pub fn new(sample_rate: u32, duration_secs: Option<f64>, tx: Sender<PlaybackSnapshot>) -> Self {
        let frames_per_stride = (sample_rate as f64 * PROGRESS_STRIDE_SECONDS).round() as u64;
        Self {
            sample_rate,
            frames_count_tracked: 0,
            frames_per_stride,
            duration_secs,
            tx,
        }
    }

    fn send_snapshot(&self, is_playing: bool) {
        let snapshot = PlaybackSnapshot {
            pos_secs: self.get_position_seconds(),
            duration_secs: self.duration_secs.unwrap_or(0.0),
            is_playing,
        };

        let _ = self.tx.send(snapshot);
    }

    fn get_position_seconds(&self) -> f64 {
        return self.frames_count_tracked as f64 / self.sample_rate as f64;
    }
}

impl SampleConsumer for PlaybackTracker {
    fn on_sample(&mut self, _sample: f32) {
        self.frames_count_tracked = self.frames_count_tracked.saturating_add(1);

        if self.frames_count_tracked % self.frames_per_stride == 0 {
            self.send_snapshot(true);
        }
    }

    fn on_state_change(&mut self, is_playing: bool) {
        self.send_snapshot(is_playing);
    }

    fn on_seek(&mut self, seek_time_secs: f64) {
        self.frames_count_tracked = (seek_time_secs * self.sample_rate as f64).round() as u64;
        self.send_snapshot(true);
    }
}
