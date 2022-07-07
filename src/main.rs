use bevy::{
    asset::AssetServerSettings,
    prelude::*,
    window::PresentMode,
};
use trace::TraceMaterial;

mod character;
mod fps_counter;
mod trace;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(WindowDescriptor {
            width: 600.0,
            height: 600.0,
            present_mode: PresentMode::Mailbox,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(fps_counter::FpsCounter)
        .add_plugin(character::Character)
        .add_plugin(trace::Tracer)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Res<Windows>,
) {
    // cube
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        TraceMaterial,
        Visibility::default(),
        ComputedVisibility::default(),
    ));

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}
