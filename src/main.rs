use bevy::{asset::AssetServerSettings, prelude::*};
use compute::{AABox, Particle};
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

fn setup(mut commands: Commands) {
    commands.spawn_bundle((
        AABox {
            material: 242,
            half_size: IVec3::new(0, 4, 4),
        },
        Transform::from_xyz(-0.5, -0.1, 0.0),
    ));
    commands.spawn_bundle((
        AABox {
            material: 242,
            half_size: IVec3::new(0, 4, 4),
        },
        Transform::from_xyz(0.5, -0.1, 0.0),
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
