use bevy::{
    asset::AssetServerSettings, prelude::*, render::view::NoFrustumCulling, window::PresentMode,
};
use trace::TraceMaterial;

mod character;
mod fps_counter;
mod load;
mod trace;
mod ui;

#[derive(Component)]
struct MainCamera;

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
        .insert_resource(load::load_vox().unwrap())
        .insert_resource(trace::ShaderTimer(Timer::from_seconds(1000.0, true)))
        .insert_resource(trace::Settings {
            show_ray_steps: false,
            freeze: false,
            misc_bool: false,
            misc_float: 20.0,
        })
        .insert_resource(trace::LastFrameData {
            last_camera: Mat4::default(),
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(fps_counter::FpsCounter)
        .add_plugin(character::Character)
        .add_plugin(trace::Tracer)
        .add_plugin(ui::UiPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Plane { size: 2.0 })),
        Transform::from_xyz(0.0, 0.0, 0.001).looking_at(Vec3::Y, Vec3::Z),
        GlobalTransform::default(),
        TraceMaterial,
        Visibility::default(),
        ComputedVisibility::default(),
        NoFrustumCulling,
    ));
}
