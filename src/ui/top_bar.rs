use egui::Context;

use crate::app_state::AppState;

use super::{helpers::pick_file_dialog, UiActions, UiEvent, View};

pub struct TopBar;

impl TopBar {
    pub fn new() -> Self {
        Self
    }
}

impl View for TopBar {
    fn ui(&self, ctx: &Context, _state: &AppState, actions: &mut UiActions) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open…").clicked() {
                        if let Some(path) = pick_file_dialog() {
                            actions.events.push(UiEvent::OpenFile(path));
                        }
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
    }
}
