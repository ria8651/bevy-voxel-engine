use bevy::prelude::*;
use super::trace;
use bevy_egui::{egui, EguiContext, EguiPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(ui_system);
    }
}

fn ui_system(mut egui_context: ResMut<EguiContext>, mut settings: ResMut<trace::Settings>) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.checkbox(&mut settings.show_ray_steps, "Show ray steps");
        });
}
