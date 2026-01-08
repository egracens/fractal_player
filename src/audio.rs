use std::fs::File;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use flume::{Receiver, Sender};
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

#[derive(Debug)]
pub enum AudioCommand {
    LoadFile(String),
    Play,
    Pause,
    Stop,
    Terminate,
}

#[derive(Clone, Debug, Default)]
pub struct PlaybackSnapshot {
    pub pos_secs: f64,
    pub duration_secs: f64,
    pub is_playing: bool,
}

pub struct AudioRuntime {
    cmd_tx: Sender<AudioCommand>,
    handle: Option<JoinHandle<()>>,
    pub progress_rx: Receiver<PlaybackSnapshot>,
}

impl AudioRuntime {
    pub fn new(initial_track: Option<String>) -> Self {
        let (cmd_tx, cmd_rx) = flume::unbounded::<AudioCommand>();
        let (progress_tx, progress_rx) = flume::bounded::<PlaybackSnapshot>(8);

        let handle = thread::Builder::new()
            .name("audio-thread".into())
            .spawn(move || audio_worker(cmd_rx, progress_tx))
            .expect("failed to spawn audio thread");

        let runtime = Self {
            cmd_tx,
            handle: Some(handle),
            progress_rx,
        };

        if let Some(path) = initial_track {
            runtime.load_file(path);
        }

        runtime
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

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new(None)
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

fn audio_worker(rx: Receiver<AudioCommand>, progress_tx: Sender<PlaybackSnapshot>) {
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
    let mut duration_secs: f64 = 0.0;
    let mut elapsed_secs: f64 = 0.0;
    let mut last_tick = Instant::now();
    let mut playing = false;

    let send_progress = |pos, dur, playing| {
        let snap = PlaybackSnapshot {
            pos_secs: pos,
            duration_secs: dur,
            is_playing: playing,
        };
        let _ = progress_tx.send(snap);
    };

    let tick = |elapsed_secs: &mut f64,
                duration_secs: f64,
                last_tick: &mut Instant,
                playing: bool,
                progress_tx: &Sender<PlaybackSnapshot>| {
        if playing {
            let dt = last_tick.elapsed().as_secs_f64();
            *elapsed_secs = (*elapsed_secs + dt).min(duration_secs.max(0.0));
            *last_tick = Instant::now();
            let _ = progress_tx.send(PlaybackSnapshot {
                pos_secs: *elapsed_secs,
                duration_secs,
                is_playing: true,
            });
        }
    };

    let timeout = Duration::from_millis(200);

    loop {
        match rx.recv_timeout(timeout) {
            Ok(cmd) => {
                // update elapsed before handling command if playing
                if playing {
                    let dt = last_tick.elapsed().as_secs_f64();
                    elapsed_secs = (elapsed_secs + dt).min(duration_secs.max(0.0));
                    last_tick = Instant::now();
                }

                match cmd {
                    AudioCommand::LoadFile(path) => {
                        log::info!("audio: load file {path}");
                        current_track = Some(path.clone());
                        elapsed_secs = 0.0;
                        duration_secs = 0.0;
                        playing = false;
                        last_tick = Instant::now();
                        if let Some(s) = sink.take() {
                            s.stop();
                        }
                        match load_sink(mixer, &path) {
                            Ok((new_sink, dur_opt)) => {
                                duration_secs = dur_opt.unwrap_or(0.0);
                                sink = Some(new_sink);
                            }
                            Err(err) => log::warn!("audio: failed to load {path}: {err}"),
                        }
                        send_progress(elapsed_secs, duration_secs, playing);
                    }
                    AudioCommand::Play => {
                        log::info!("audio: play");
                        if let Some(s) = sink.as_ref() {
                            playing = true;
                            last_tick = Instant::now();
                            s.play();
                            send_progress(elapsed_secs, duration_secs, playing);
                        }
                    }
                    AudioCommand::Pause => {
                        log::info!("audio: pause");
                        if let Some(s) = sink.as_ref() {
                            playing = false;
                            s.pause();
                            send_progress(elapsed_secs, duration_secs, playing);
                        }
                    }
                    AudioCommand::Stop => {
                        log::info!("audio: stop");
                        if let Some(track) = current_track.clone() {
                            elapsed_secs = 0.0;
                            playing = false;
                            last_tick = Instant::now();
                            let (new_sink, dur_opt) = load_sink_on_position(Duration::ZERO, track, mixer);
                            sink = new_sink;
                            duration_secs = dur_opt.unwrap_or(0.0);
                            send_progress(elapsed_secs, duration_secs, playing);
                        }
                    }
                    AudioCommand::Terminate => {
                        log::info!("audio: terminate");
                        if let Some(s) = sink.take() {
                            s.stop();
                        }
                        send_progress(elapsed_secs, duration_secs, false);
                        break;
                    }
                }
            }
            Err(flume::RecvTimeoutError::Timeout) => {
                tick(&mut elapsed_secs, duration_secs, &mut last_tick, playing, &progress_tx);
            }
            Err(flume::RecvTimeoutError::Disconnected) => break,
        }
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
