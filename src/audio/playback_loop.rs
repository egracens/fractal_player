use std::fs::File;
use std::time::Duration;

use flume::Sender;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

use crate::audio::{
    AnalyzerBins, AudioCommand, FFT_SIZE, FFTProcessor, PlaybackSnapshot, PlaybackTracker,
    SampleFanout, SpectrogramBins,
};

pub struct PlaybackLoop {
    _stream: rodio::OutputStream,
    mixer: rodio::mixer::Mixer,
    sink: Option<Sink>,
    current_track: Option<String>,
    duration_secs: f64,
    progress_tx: Sender<PlaybackSnapshot>,
    spectrogram_tx: Sender<SpectrogramBins>,
}

impl PlaybackLoop {
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

    fn play(&mut self) {
        if let Some(s) = self.sink.as_ref() {
            s.play();
        }
    }

    fn pause(&mut self) {
        if let Some(s) = self.sink.as_ref() {
            s.pause();
        }
    }

    fn stop(&mut self) {
        if let Some(track) = self.current_track.clone() {
            let (new_sink, dur_opt) = self.load_sink_on_position(Duration::ZERO, track);
            self.sink = new_sink;
            self.duration_secs = dur_opt.unwrap_or(0.0);
        }
    }

    fn terminate(&mut self) {
        if let Some(s) = self.sink.take() {
            s.stop();
        }
    }

    fn load_sink_on_position(
        &mut self,
        _pos: Duration,
        track: String,
    ) -> (Option<Sink>, Option<f64>) {
        match self.load_sink(&track) {
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
            AnalyzerBins::new(FFT_SIZE),
            self.spectrogram_tx.clone(),
        );
        let fanout = SampleFanout::new(decoder, tracker, fft_proc);

        let sink = Sink::connect_new(&self.mixer);
        sink.pause();
        sink.append(fanout);
        Ok((sink, duration_opt))
    }
}
