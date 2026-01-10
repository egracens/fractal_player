use std::fs::File;
use std::time::Duration;

use flume::Sender;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

use crate::audio::{
    AudioCommand, DefaultAnalyzer, FFTProcessor, PlaybackSnapshot, PlaybackTracker, SampleConsumer,
    SampleFanout, SpectrogramBins,
};

pub struct AudioWorker {
    _stream: rodio::OutputStream,
    mixer: rodio::mixer::Mixer,
    sink: Option<Sink>,
    current_track: Option<String>,
    duration_secs: f64,
    progress_tx: Sender<PlaybackSnapshot>,
    spectrogram_tx: Sender<SpectrogramBins>,
    state_tx: Option<Sender<bool>>,
}

impl AudioWorker {
    pub fn new(
        progress_tx: Sender<PlaybackSnapshot>,
        spectrogram_tx: Sender<SpectrogramBins>,
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
            progress_tx,
            spectrogram_tx,
            state_tx: None,
        })
    }

    pub fn handle_command(&mut self, cmd: AudioCommand) {
        match cmd {
            AudioCommand::LoadFile(path) => self.load(path),
            AudioCommand::Play => self.play(),
            AudioCommand::Pause => self.pause(),
            AudioCommand::Stop => self.stop(),
            AudioCommand::Terminate => self.terminate(),
        }
    }

    fn load(&mut self, path: String) {
        self.current_track = Some(path.clone());
        self.duration_secs = 0.0;

        if let Some(s) = self.sink.take() {
            s.stop();
        }

        match self.load_sink(&path) {
            Ok((new_sink, dur_opt)) => {
                self.duration_secs = dur_opt.unwrap_or(0.0);
                self.sink = Some(new_sink);
            }
            Err(err) => {
                log::warn!("audio: failed to load {path}: {err}");
            }
        }
    }

    fn play(&self) {
        if let Some(s) = self.sink.as_ref() {
            s.play();
        }
        if let Some(tx) = &self.state_tx {
            let _ = tx.send(true);
        }
    }

    fn pause(&self) {
        if let Some(s) = self.sink.as_ref() {
            s.pause();
        }
        if let Some(tx) = &self.state_tx {
            let _ = tx.send(false);
        }
    }

    fn stop(&mut self) {
        if let Some(track) = self.current_track.clone() {
            let (new_sink, dur_opt) = self.load_sink_at_position(&track, Duration::ZERO);
            self.sink = new_sink;
            self.duration_secs = dur_opt.unwrap_or(0.0);
        }
    }

    fn terminate(&mut self) {
        if let Some(s) = self.sink.take() {
            s.stop();
        }
    }

    fn load_sink_at_position(
        &mut self,
        track: &str,
        _pos: Duration,
    ) -> (Option<Sink>, Option<f64>) {
        match self.load_sink(track) {
            Ok((sink, dur_opt)) => (Some(sink), dur_opt),
            Err(err) => {
                log::warn!("audio: failed to reset {track}: {err}");
                (None, None)
            }
        }
    }

    fn load_sink(&mut self, path: &str) -> Result<(Sink, Option<f64>), String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let decoder = Decoder::try_from(file).map_err(|e| e.to_string())?;
        let duration_opt = decoder.total_duration().map(|d| d.as_secs_f64());
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();

        let tracker = PlaybackTracker::new(sample_rate, duration_opt, self.progress_tx.clone());
        let fft_proc = FFTProcessor::new(
            channels,
            DefaultAnalyzer::new(1024),
            self.spectrogram_tx.clone(),
            sample_rate,
        );

        let consumers: Vec<Box<dyn SampleConsumer>> = vec![Box::new(tracker), Box::new(fft_proc)];

        let (state_tx, state_rx) = flume::bounded::<bool>(1);
        self.state_tx = Some(state_tx);

        let fanout = SampleFanout::with_state_channel(decoder, consumers, state_rx);

        let sink = Sink::connect_new(&self.mixer);
        sink.pause();
        sink.append(fanout);
        Ok((sink, duration_opt))
    }
}
