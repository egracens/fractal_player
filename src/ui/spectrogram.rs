use egui::{Context, Ui};
use egui_plot::{Bar, BarChart, Plot};

use crate::{app_state::AppState, audio::SpectrogramSlice};

use super::{UiActions, View};

pub struct Spectrogram;

impl Spectrogram {
    pub fn new() -> Self {
        Self
    }

    fn draw_latest_slice(&self, ui: &mut Ui, state: &AppState) {
        const Y_MAX: f64 = 5.0;

        if let Some(SpectrogramSlice {
            bins,
            sample_rate_hz,
            window_size,
            ..
        }) = state.spectrogram.last()
        {
            // Calculate frequency spacing between bins
            // Each output bin covers 2 FFT bins, so spacing = sample_rate / window_size * 2
            let bin_spacing = *sample_rate_hz as f64 / *window_size as f64 * 2.0;

            let bars: Vec<Bar> = bins
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mag = (*v as f64).min(Y_MAX);
                    // Convert bin index to frequency (Hz)
                    // Each output bin covers 2 FFT bins: bin i covers FFT bins [i*2, i*2+1]
                    // Center frequency ≈ (i * 2 + 0.5) * sample_rate / window_size
                    let frequency =
                        (i as f64 * 2.0 + 0.5) * *sample_rate_hz as f64 / *window_size as f64;
                    Bar::new(frequency, mag).width(bin_spacing)
                })
                .collect();

            let chart = BarChart::new("spectrogram_latest", bars).color(egui::Color32::LIGHT_BLUE);

            // Calculate max frequency (Nyquist frequency = sample_rate / 2)
            let max_freq = *sample_rate_hz as f64 / 2.0;

            Plot::new("spectrogram_plot")
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_drag(false)
                .include_y(0.0)
                .include_y(Y_MAX)
                .include_x(0.0)
                .include_x(max_freq)
                .height(200.0)
                .show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);
                });
        } else {
            ui.label("Waiting for spectrogram data…");
        }
    }
}

impl View for Spectrogram {
    fn ui(&self, ctx: &Context, state: &AppState, _actions: &mut UiActions) {
        egui::Window::new("Spectrogram")
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                self.draw_latest_slice(ui, state);
            });
    }
}
