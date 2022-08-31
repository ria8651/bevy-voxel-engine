use crate::animation::Velocity;

use super::trace;
use super::{Bullet, Particle};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use egui::Slider;
use rand::Rng;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(ui_system);
    }
}

fn ui_system(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut uniforms: ResMut<trace::Uniforms>,
    mut settings: ResMut<super::Settings>,
    particle_query: Query<Entity, With<Particle>>,
) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.collapsing("Rendering", |ui| {
                ui.checkbox(&mut uniforms.show_ray_steps, "Show ray steps");
                ui.checkbox(&mut uniforms.indirect_lighting, "Indirect lighting");
                ui.checkbox(&mut uniforms.shadows, "Shadows");
                ui.checkbox(&mut uniforms.skybox, "Skybox");
                ui.add(
                    Slider::new(&mut uniforms.accumulation_frames, 1.0..=100.0)
                        .text("Accumulation frames"),
                );
                ui.checkbox(&mut uniforms.freeze, "Freeze");
            });
            ui.collapsing("Compute", |ui| {
                ui.checkbox(&mut uniforms.enable_compute, "Enable compute");
                if ui.button("spawn particles").clicked() {
                    let mut rng = rand::thread_rng();
                    for _ in 0..10000 {
                        commands.spawn_bundle((
                            Transform::from_xyz(0.0, 0.0, 0.0),
                            Particle { material: rng.gen_range(100..104) },
                            Velocity {
                                velocity: Vec3::new(
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                ).clamp_length_max(1.0) * 10.0,
                            },
                            Bullet,
                        ));
                    }
                }
                if ui.button("destroy particles").clicked() {
                    for particle in particle_query.iter() {
                        commands.entity(particle).despawn();
                    }
                }
                ui.label(format!("Particle count: {}", particle_query.iter().count()));
            });
            ui.checkbox(&mut settings.spectator, "Spectator mode");
            ui.checkbox(&mut uniforms.misc_bool, "Misc bool");
            ui.add(Slider::new(&mut uniforms.misc_float, 0.0..=1.0).text("Misc float"));
        });
}
