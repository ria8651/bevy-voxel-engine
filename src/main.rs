use bevy::{asset::AssetServerSettings, prelude::*, render::view::NoFrustumCulling};
use compute::Particle;
use trace::TraceMaterial;

mod character;
mod compute;
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
            ..default()
        })
        .insert_resource(load::load_vox().unwrap())
        .insert_resource(trace::ShaderTimer(Timer::from_seconds(1000.0, true)))
        .insert_resource(trace::LastFrameData {
            last_camera: Mat4::default(),
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(fps_counter::FpsCounter)
        .add_plugin(character::Character)
        .add_plugin(trace::Tracer)
        .add_plugin(ui::UiPlugin)
        .add_plugin(compute::ComputePlugin)
        .add_startup_system(setup)
        .add_system(update_particles)
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

    commands.spawn_bundle((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Particle { material: 41 },
    ));
    commands.spawn_bundle((
        Transform::from_xyz(0.1, 0.0, 0.0),
        Particle { material: 41 },
    ));
}

fn update_particles(mut particle_query: Query<&mut Transform, With<Particle>>, time: Res<Time>) {
    for mut particle in particle_query.iter_mut() {
        particle.translation += Vec3::new(
            time.seconds_since_startup().sin() as f32 / 1000.0,
            time.seconds_since_startup().cos() as f32 / 1000.0,
            0.0,
        );
    }
}
