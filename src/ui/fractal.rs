use egui::{Context, Ui};

use crate::{
    app_state::{AppState, FractalType},
    audio::SpectrogramBins,
    ui::fractal_callbacks::{AuroraCallback, JuliaCallback, MandelbrotCallback, TriangleCallback},
};

use super::{UiActions, View};

pub struct Fractal;

impl Fractal {
    pub fn new() -> Self {
        Self
    }
}

impl View for Fractal {
    fn ui(&self, ctx: &Context, state: &AppState, actions: &mut UiActions) {
        egui::Window::new("Fractal")
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                self.draw_fractal(ctx, ui, state, actions);
            });
    }
}

impl Fractal {
    fn draw_fractal(&self, _ctx: &Context, ui: &mut Ui, state: &AppState, actions: &mut UiActions) {
        ui.horizontal(|ui| {
            ui.label("Fractal Type:");

            // Dropdown to select fractal type
            let mut selected_type = state.fractal_type;
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", selected_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_type, FractalType::Triangle, "Triangle");
                    ui.selectable_value(&mut selected_type, FractalType::Aurora, "Aurora");
                    ui.selectable_value(&mut selected_type, FractalType::Mandelbrot, "Mandelbrot");
                    ui.selectable_value(&mut selected_type, FractalType::Julia, "Julia");
                });

            // Send event if selection changed
            if selected_type != state.fractal_type {
                actions
                    .events
                    .push(crate::ui::UiEvent::ChangeFractal(selected_type));
            }
        });

        ui.separator();

        let desired_size = egui::vec2(ui.available_width(), ui.available_height().max(1.0));
        let (canvas_rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        // Get FFT data
        let fft_data = if state.is_playing {
            state.spectrogram.last().copied().unwrap_or_default()
        } else {
            SpectrogramBins::default()
        };

        let current_time = state.visual_time;

        let target_format = state
            .target_format
            .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

        // Create callback based on selected fractal type
        let callback = match state.fractal_type {
            FractalType::Triangle => eframe::egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                TriangleCallback::new(fft_data, current_time, target_format),
            ),
            FractalType::Aurora => eframe::egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                AuroraCallback::new(fft_data, current_time, target_format),
            ),
            FractalType::Mandelbrot => eframe::egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                MandelbrotCallback::new(fft_data, current_time, target_format),
            ),
            FractalType::Julia => eframe::egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                JuliaCallback::new(fft_data, current_time, target_format),
            ),
        };

        ui.painter().add(egui::Shape::Callback(callback));
    }
}
