use super::{character::CharacterEntity, Bullet, Particle, VoxelizationPreviewCamera};
use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa, tonemapping::Tonemapping},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts, EguiPlugin,
};
use bevy_voxel_engine::{
    Flags, LoadVoxelWorld, RenderGraphSettings, TraceSettings, VoxelPhysics,
};
use rand::Rng;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_system);
    }
}

fn ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    particle_query: Query<Entity, (With<VoxelPhysics>, Without<CharacterEntity>)>,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut render_graph_settings: ResMut<RenderGraphSettings>,
    mut camera_settings_query: Query<(
        &mut TraceSettings,
        Option<&mut BloomSettings>,
        Option<&mut Tonemapping>,
        Option<&mut Fxaa>,
    )>,
    mut voxelization_preview_camera_query: Query<&mut Camera, With<VoxelizationPreviewCamera>>,
    mut character_query: Query<&mut CharacterEntity>,
) {
    let mut character = character_query.single_mut();

    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(contexts.ctx_mut(), |ui| {
            if ui.button("Open File").clicked() {
                // let path = rfd::AsyncFileDialog::new()
                //     .add_filter("Magica Voxel VOX File", &["vox"])
                //     .pick_file().block_on();

                let path = tinyfiledialogs::open_file_dialog("Select file", "", None);
                *load_voxel_world = LoadVoxelWorld::File(path.unwrap());
            }
            for (i, (mut trace_settings, bloom_settings, tonemapping, fxaa)) in
                camera_settings_query.iter_mut().enumerate()
            {
                ui.collapsing(format!("Camera Settings {}", i), |ui| {
                    ui.checkbox(&mut trace_settings.show_ray_steps, "Show ray steps");
                    ui.add(Slider::new(&mut trace_settings.samples, 1..=8).text("Samples"));
                    ui.checkbox(&mut trace_settings.shadows, "Shadows");
                    if let Some(bloom_settings) = bloom_settings {
                        ui.add(
                            Slider::new(&mut bloom_settings.into_inner().intensity, 0.0..=1.0)
                                .text("Bloom"),
                        );
                    }
                    if let Some(mut tonemapping) = tonemapping {
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", tonemapping.as_mut()))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::AcesFitted,
                                    "AcesFitted",
                                );
                                ui.selectable_value(tonemapping.as_mut(), Tonemapping::AgX, "AgX");
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::BlenderFilmic,
                                    "BlenderFilmic",
                                );
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::Reinhard,
                                    "Reinhard",
                                );
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::ReinhardLuminance,
                                    "ReinhardLuminance",
                                );
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::SomewhatBoringDisplayTransform,
                                    "SomewhatBoringDisplayTransform",
                                );
                                ui.selectable_value(
                                    tonemapping.as_mut(),
                                    Tonemapping::None,
                                    "None",
                                );
                            });
                    }
                    if let Some(fxaa) = fxaa {
                        ui.checkbox(&mut fxaa.into_inner().enabled, "FXAA");
                    }
                });
            }
            ui.collapsing("Compute", |ui| {
                if ui.button("spawn particles").clicked() {
                    let mut rng = rand::thread_rng();
                    for _ in 0..10 {
                        commands.spawn((
                            Transform::from_xyz(0.0, 0.0, 0.0),
                            Particle {
                                material: rng.gen_range(100..104),
                                flags: Flags::ANIMATION_FLAG,
                            },
                            VoxelPhysics::new(
                                Vec3::new(
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                    rng.gen_range(-1.0..1.0),
                                )
                                .clamp_length_max(1.0)
                                    * 10.0,
                                Vec3::new(0.0, -9.81, 0.0),
                                bevy_voxel_engine::CollisionEffect::None,
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
            
            ui.collapsing("Passes", |ui| {
                ui.checkbox(&mut render_graph_settings.clear, "clear");
                ui.checkbox(&mut render_graph_settings.automata, "automata");
                ui.checkbox(&mut render_graph_settings.animation, "animation");
                ui.checkbox(&mut render_graph_settings.voxelization, "voxelization");
                ui.checkbox(&mut render_graph_settings.rebuild, "rebuild");
                ui.checkbox(&mut render_graph_settings.physics, "physics");
                ui.checkbox(&mut render_graph_settings.trace, "trace");
            });

            for mut voxelization_preview_camera in voxelization_preview_camera_query.iter_mut() {
                ui.checkbox(
                    &mut voxelization_preview_camera.is_active,
                    format!("Preview"),
                );
            }
            ui.checkbox(&mut character.in_spectator, "Spectator mode");
        });
}
