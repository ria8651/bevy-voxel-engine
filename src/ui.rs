use crate::VoxelizationPreviewCamera;

use super::character::CharacterEntity;
use super::{Bullet, Particle, Settings, Velocity};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::fxaa::Fxaa;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Slider},
    EguiContext, EguiPlugin,
};
use bevy_voxel_engine::{DenoiseSettings, LoadVoxelWorld, RenderGraphSettings, TraceSettings};
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
    mut settings: ResMut<Settings>,
    particle_query: Query<Entity, (With<Velocity>, Without<CharacterEntity>)>,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut render_graph_settings: ResMut<RenderGraphSettings>,
    mut camera_settings_query: Query<(
        &mut TraceSettings,
        Option<&mut BloomSettings>,
        Option<&mut Fxaa>,
    )>,
    mut denoise_pass_data: ResMut<DenoiseSettings>,
    mut voxelization_preview_camera_query: Query<&mut Camera, With<VoxelizationPreviewCamera>>,
) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(egui_context.ctx_mut(), |ui| {
            if ui.button("Open File").clicked() {
                // let path = rfd::AsyncFileDialog::new()
                //     .add_filter("Magica Voxel VOX File", &["vox"])
                //     .pick_file().block_on();

                let path = tinyfiledialogs::open_file_dialog("Select file", "", None);
                *load_voxel_world = LoadVoxelWorld::File(path.unwrap());
            }
            for (i, (mut trace_settings, bloom_settings, fxaa)) in
                camera_settings_query.iter_mut().enumerate()
            {
                ui.collapsing(format!("Camera Settings {}", i), |ui| {
                    ui.checkbox(&mut trace_settings.show_ray_steps, "Show ray steps");
                    ui.checkbox(&mut trace_settings.indirect_lighting, "Indirect lighting");
                    ui.add(Slider::new(&mut trace_settings.samples, 1..=8).text("Samples"));
                    ui.add(
                        Slider::new(&mut trace_settings.reprojection_factor, 0.0..=1.0)
                            .text("Reprojection"),
                    );
                    ui.checkbox(&mut trace_settings.shadows, "Shadows");
                    ui.checkbox(&mut trace_settings.misc_bool, "Misc");
                    ui.add(Slider::new(&mut trace_settings.misc_float, 0.0..=1.0).text("Misc"));
                    if let Some(bloom_settings) = bloom_settings {
                        ui.add(Slider::new(&mut bloom_settings.into_inner().intensity, 0.0..=1.0).text("Bloom"));
                    }
                    if let Some(fxaa) = fxaa {
                        ui.checkbox(&mut fxaa.into_inner().enabled, "FXAA");
                    }
                });
            }
            ui.collapsing("Compute", |ui| {
                if ui.button("spawn particles").clicked() {
                    let mut rng = rand::thread_rng();
                    for _ in 0..10000 {
                        commands.spawn((
                            Transform::from_xyz(0.0, 0.0, 0.0),
                            Particle {
                                material: rng.gen_range(100..104),
                            },
                            Velocity::new(
                                Vec3::new(
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                )
                                .clamp_length_max(1.0)
                                    * 10.0,
                            ),
                            Bullet { bullet_type: 0 },
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
            ui.collapsing("Denoise", |ui| {
                for i in 0..3 {
                    ui.label(format!("Pass {}", i));
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].denoise_strength,
                            0.0..=8.0,
                        )
                        .text("Strength"),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].colour_phi,
                            0.01..=1.0,
                        )
                        .text("Colour")
                        .logarithmic(true),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].normal_phi,
                            0.1..=100.0,
                        )
                        .text("Normal")
                        .logarithmic(true),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].position_phi,
                            0.01..=1.0,
                        )
                        .text("Position")
                        .logarithmic(true),
                    );
                }
            });
            ui.collapsing("Passes", |ui| {
                ui.checkbox(&mut render_graph_settings.clear, "clear");
                ui.checkbox(&mut render_graph_settings.automata, "automata");
                ui.checkbox(&mut render_graph_settings.voxelization, "voxelization");
                ui.checkbox(&mut render_graph_settings.rebuild, "rebuild");
                ui.checkbox(&mut render_graph_settings.physics, "physics");
                ui.checkbox(&mut render_graph_settings.trace, "trace");
                ui.checkbox(&mut render_graph_settings.denoise, "denoise");
            });

            for mut voxelization_preview_camera in voxelization_preview_camera_query.iter_mut() {
                ui.checkbox(
                    &mut voxelization_preview_camera.is_active,
                    format!("Preview"),
                );
            }
            ui.checkbox(&mut settings.spectator, "Spectator mode");
        });
}
