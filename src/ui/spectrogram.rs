use egui::Context;
use egui_plot::{Bar, BarChart, Plot};

use crate::{app_state::AppState, audio::SpectrogramSlice};

use super::{UiActions, View};

pub struct Spectrogram;

impl Spectrogram {
    pub fn new() -> Self {
        Self
    }
}

impl View for Spectrogram {
    fn ui(&self, ctx: &Context, state: &AppState, _actions: &mut UiActions) {
        egui::Window::new("Dummy spectrogram")
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                if let Some(SpectrogramSlice { bins }) = state.spectrogram.last() {
                    let bars: Vec<Bar> = bins
                        .iter()
                        .enumerate()
                        .map(|(i, v)| Bar::new(i as f64, *v as f64).width(0.9))
                        .collect();

                    let chart =
                        BarChart::new("spectrogram_latest", bars).color(egui::Color32::LIGHT_BLUE);

                    Plot::new("spectrogram_plot")
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .allow_drag(false)
                        .include_y(0.0)
                        .height(200.0)
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(chart);
                        });
                } else {
                    ui.label("Waiting for spectrogram data…");
                }
            });
    }
}
