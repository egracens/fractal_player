use eframe::egui_wgpu::CallbackTrait;
use egui::{Context, Ui};

use crate::{
    app_state::{AppState, FractalType},
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
                self.draw_fractal(ui, state);
            });
    }
}

impl Fractal {
    fn draw_fractal(&self, ui: &mut Ui, state: &AppState) {
        ui.horizontal(|ui| {
            ui.label("Fractal:");
            ui.label(format!("{:?}", state.fractal_type));
        });

        ui.separator();

        let desired_size = egui::vec2(ui.available_width(), ui.available_height().max(1.0));
        let (canvas_rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            canvas_rect,
            Self::get_callback_for_fractal_type(state.fractal_type),
        );

        ui.painter().add(egui::Shape::Callback(callback));
    }

    fn get_callback_for_fractal_type(fractal_type: FractalType) -> impl CallbackTrait {
        match fractal_type {
            FractalType::Triangle => TriangleCallback::new(),
        }
    }
}
