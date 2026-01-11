use egui::{Context, Ui};

use crate::{
    app_state::AppState,
    audio::SpectrogramBins,
    ui::fractal_callbacks::TriangleCallback,
};

use super::{UiActions, View};

pub struct Fractal;

impl Fractal {
    pub fn new() -> Self {
        Self
    }
}

impl View for Fractal {
    fn ui(&self, ctx: &Context, state: &AppState, _actions: &mut UiActions) {
        egui::Window::new("Fractal")
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                self.draw_fractal(ctx, ui, state);
            });
    }
}

impl Fractal {
    fn draw_fractal(&self, ctx: &Context, ui: &mut Ui, state: &AppState) {
        ui.horizontal(|ui| {
            ui.label("Fractal:");
            ui.label(format!("{:?}", state.fractal_type));
        });

        ui.separator();

        let desired_size = egui::vec2(ui.available_width(), ui.available_height().max(1.0));
        let (canvas_rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        // Get FFT data and create triangle callback
        let fft_data = if state.is_playing {
            state.spectrogram.last().copied().unwrap_or_default()
        } else {
            SpectrogramBins::default()
        };

        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            canvas_rect,
            TriangleCallback::new(fft_data, ctx.input(|i| i.time)),
        );

        ui.painter().add(egui::Shape::Callback(callback));
    }
}
