use bevy::{asset::AssetServerSettings, prelude::*};
use character::CharacterEntity;
use compute::{Edges, Particle, Portal, Bullet};
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
        // .add_startup_system(setup)
        .add_system(shoot)
        // .add_system(update_particles)
        .run();
}

fn shoot(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    character: Query<&Transform, With<CharacterEntity>>,
) {
    let character = character.single();

    if input.just_pressed(MouseButton::Left) {
        commands.spawn_bundle((
            Transform::from_translation(character.translation).with_rotation(character.rotation),
            Particle { material: 41 },
            Bullet { velocity: -character.local_z() * 10.0 },
        ));
    }
}

// world space cordinates are in terms of 4 voxels per meter with 0, 0
// in the world lining up with the center of the voxel world (ie 0, 0, 0 in the render world)
// fn setup(mut commands: Commands) {
//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(4, 4, 0),
//             normal: Vec3::new(0.0, 0.0, 1.0),
//         },
//         Edges {
//             material: 23,
//             half_size: IVec3::new(5, 5, 0),
//         },
//         Transform::from_xyz(3.0, 5.0, 0.0),
//     ));
//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(4, 4, 0),
//             normal: Vec3::new(0.0, 0.0, 1.0),
//         },
//         Edges {
//             material: 22,
//             half_size: IVec3::new(5, 5, 0),
//         },
//         Transform::from_xyz(-3.0, 5.0, 0.0),
//     ));

//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(0, 4, 4),
//             normal: Vec3::new(1.0, 0.0, 0.0),
//         },
//         Edges {
//             material: 23,
//             half_size: IVec3::new(0, 5, 5),
//         },
//         Transform::from_xyz(-3.0, 2.0, 0.0),
//     ));
//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(4, 4, 0),
//             normal: Vec3::new(0.0, 0.0, 1.0),
//         },
//         Edges {
//             material: 22,
//             half_size: IVec3::new(5, 5, 0),
//         },
//         Transform::from_xyz(0.0, 2.0, -3.0),
//     ));

//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(4, 4, 0),
//             normal: Vec3::new(0.0, 0.0, -1.0),
//         },
//         Edges {
//             material: 23,
//             half_size: IVec3::new(5, 5, 0),
//         },
//         Transform::from_xyz(0.0, 8.0, 3.0),
//     ));
//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(4, 4, 0),
//             normal: Vec3::new(0.0, 0.0, 1.0),
//         },
//         Edges {
//             material: 22,
//             half_size: IVec3::new(5, 5, 0),
//         },
//         Transform::from_xyz(0.0, 8.0, -3.0),
//     ));

//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(0, 4, 4),
//             normal: Vec3::new(-1.0, 0.0, 0.0),
//         },
//         Edges {
//             material: 23,
//             half_size: IVec3::new(0, 5, 5),
//         },
//         Transform::from_xyz(3.0, 8.0, 0.0),
//     ));
//     commands.spawn_bundle((
//         Portal {
//             material: 1,
//             half_size: IVec3::new(0, 4, 4),
//             normal: Vec3::new(1.0, 0.0, 0.0),
//         },
//         Edges {
//             material: 22,
//             half_size: IVec3::new(0, 5, 5),
//         },
//         Transform::from_xyz(-3.0, 8.0, 0.0),
//     ));
// }

// fn update_particles(mut particle_query: Query<&mut Transform, With<Particle>>) {
//     particle_query.par_for_each_mut(32, |mut particle| {
//         let mut rng = rand::thread_rng();
//         particle.translation += Vec3::new(
//             rng.gen_range(-1.0..=1.0) * 0.25,
//             rng.gen_range(-1.0..=1.0) * 0.25,
//             rng.gen_range(-1.0..=1.0) * 0.25,
//         );
//     });
// }
