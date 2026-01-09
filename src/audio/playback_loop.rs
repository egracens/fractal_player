use std::fs::File;
use std::time::{Duration, Instant};

use flume::Sender;
use rand::Rng;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

use crate::audio::{AudioCommand, PlaybackSnapshot, SpectrogramSlice};

pub struct PlaybackLoop {
    _stream: rodio::OutputStream,
    mixer: rodio::mixer::Mixer,
    sink: Option<Sink>,
    current_track: Option<String>,
    duration_secs: f64,
    elapsed_secs: f64,
    last_tick: Instant,
    playing: bool,
    progress_tx: Sender<PlaybackSnapshot>,
    spectrogram_tx: Sender<SpectrogramSlice>,
}

impl PlaybackLoop {
    pub fn new(
        progress_tx: Sender<PlaybackSnapshot>,
        spectrogram_tx: Sender<SpectrogramSlice>,
    ) -> Result<Self, String> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("failed to open output stream: {e}"))?;
        let mixer = stream.mixer().clone();

        Ok(Self {
            _stream: stream,
            mixer,
            sink: None,
            current_track: None,
            duration_secs: 0.0,
            elapsed_secs: 0.0,
            last_tick: Instant::now(),
            playing: false,
            progress_tx,
            spectrogram_tx,
        })
    }

    pub fn handle_command(&mut self, cmd: AudioCommand) {
        self.advance_elapsed();

        match cmd {
            AudioCommand::LoadFile(path) => self.load(path),
            AudioCommand::Play => self.play(),
            AudioCommand::Pause => self.pause(),
            AudioCommand::Stop => self.stop(),
            AudioCommand::Terminate => self.terminate(),
        }
    }

    pub fn tick(&mut self) {
        self.advance_elapsed();
        self.emit_dummy_spectrogram();
    }

    fn advance_elapsed(&mut self) {
        if !self.playing {
            return;
        }

        let dt = self.last_tick.elapsed().as_secs_f64();
        self.elapsed_secs = (self.elapsed_secs + dt).min(self.duration_secs.max(0.0));
        self.last_tick = Instant::now();

        self.emit_playback_progress();
    }

    fn emit_playback_progress(&self) {
        let snapshot = PlaybackSnapshot {
            pos_secs: self.elapsed_secs,
            duration_secs: self.duration_secs,
            is_playing: self.playing,
        };

        let _ = self.progress_tx.send(snapshot);
    }

    fn emit_dummy_spectrogram(&self) {
        let mut rng = rand::thread_rng();
        let mut bins = [0.0f32; 32];
        for v in bins.iter_mut() {
            let raw: f32 = rng.gen_range(0.0..1.0);
            *v = raw;
        }
        let _ = self.spectrogram_tx.send(SpectrogramSlice::new(bins));
    }

    fn load(&mut self, path: String) {
        self.current_track = Some(path.clone());
        self.elapsed_secs = 0.0;
        self.duration_secs = 0.0;
        self.playing = false;
        self.last_tick = Instant::now();

        if let Some(s) = self.sink.take() {
            s.stop();
        }

        match load_sink(&self.mixer, &path) {
            Ok((new_sink, dur_opt)) => {
                self.duration_secs = dur_opt.unwrap_or(0.0);
                self.sink = Some(new_sink);
            }
            Err(err) => {
                log::warn!("audio: failed to load {path}: {err}");
            }
        }

        self.emit_playback_progress();
    }

    fn play(&mut self) {
        if let Some(s) = self.sink.as_ref() {
            self.playing = true;
            self.last_tick = Instant::now();
            s.play();
            self.emit_playback_progress();
        }
    }

    fn pause(&mut self) {
        if let Some(s) = self.sink.as_ref() {
            self.playing = false;
            s.pause();
            self.emit_playback_progress();
        }
    }

    fn stop(&mut self) {
        if let Some(track) = self.current_track.clone() {
            self.elapsed_secs = 0.0;
            self.playing = false;
            self.last_tick = Instant::now();
            let (new_sink, dur_opt) = load_sink_on_position(Duration::ZERO, track, &self.mixer);
            self.sink = new_sink;
            self.duration_secs = dur_opt.unwrap_or(0.0);
            self.emit_playback_progress();
        }
    }

    fn terminate(&mut self) {
        if let Some(s) = self.sink.take() {
            s.stop();
        }
        self.playing = false;
        self.emit_playback_progress();
    }
}

fn load_sink_on_position(
    pos: Duration,
    track: String,
    mixer: &rodio::mixer::Mixer,
) -> (Option<Sink>, Option<f64>) {
    match load_sink(mixer, &track) {
        Ok((sink, dur_opt)) => {
            if pos > Duration::ZERO {
                let _ = sink.try_seek(pos);
            }
            (Some(sink), dur_opt)
        }
        Err(err) => {
            log::warn!("audio: failed to reset {track}: {err}");
            (None, None)
        }
    }
}

fn load_sink(mixer: &rodio::mixer::Mixer, path: &str) -> Result<(Sink, Option<f64>), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let decoder = Decoder::try_from(file).map_err(|e| e.to_string())?;
    let duration_opt = decoder.total_duration().map(|d| d.as_secs_f64());
    let sink = Sink::connect_new(mixer);
    sink.pause();
    sink.append(decoder);
    Ok((sink, duration_opt))
}
