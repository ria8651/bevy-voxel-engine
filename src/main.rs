use bevy::{asset::AssetServerSettings, prelude::*, render::view::NoFrustumCulling};
use compute::Particle;
use trace::TraceMaterial;
use rand::Rng;

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
}

fn update_particles(mut particle_query: Query<&mut Transform, With<Particle>>) {
    particle_query.par_for_each_mut(32, move |mut particle| {
        let mut rng = rand::thread_rng();
        particle.translation += Vec3::new(
            rng.gen_range(-1.0..=1.0) * 0.01,
            rng.gen_range(-1.0..=1.0) * 0.01,
            rng.gen_range(-1.0..=1.0) * 0.01,
        );
    });
}
