use bevy::{
    core_pipeline::fxaa::Fxaa,
    prelude::*,
};
use bevy_voxel_engine::{
    BevyVoxelEnginePlugin, Edges, Flags, LoadVoxelWorld, Portal, VoxelCameraBundle,
    VoxelPhysics, CollisionEffect, BoxCollider,
};
use std::f32::consts::PI;
use character::CharacterEntity;

#[path = "common/fps_counter.rs"]
mod fps_counter;

#[path = "common/character.rs"]
mod character;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyVoxelEnginePlugin)
        .add_plugins(fps_counter::FpsCounter)
        .add_plugins(character::Character)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut _meshes: ResMut<Assets<Mesh>>,
) {
    // Voxel world
    *load_voxel_world = LoadVoxelWorld::File("assets/monu9.vox".to_string());

    // character
    let character_transform = Transform::from_xyz(5.0, 5.0, -5.0)
        .looking_at(Vec3::ZERO, Vec3::Y);

    let projection = Projection::Perspective(PerspectiveProjection {
        fov: PI / 2.0,
        ..default()
    });

    // camera
    commands.spawn((
        VoxelCameraBundle {
            transform: character_transform,
            projection: projection.clone(),
            ..default()
        },
        CharacterEntity {
            in_spectator: true,
            grounded: false,
            look_at: -character_transform.local_z(),
            up: Vec3::new(0.0, 1.0, 0.0),
        },
        VoxelPhysics::new(
            Vec3::splat(0.0),
            Vec3::ZERO, // gravity handeled in character.rs
            CollisionEffect::None,
        ),
        BoxCollider {
            half_size: IVec3::new(2, 4, 2),
        },
        Fxaa::default(),
    ));

    /*
    // portal pair
    commands.spawn((
        Portal,
        Edges {
            material: 23,
            flags: Flags::ANIMATION_FLAG,
            half_size: IVec3::new(0, 10, 7),
        },
        Transform::from_xyz(-5.0, 0.0, -5.0),
    ));
    commands.spawn((
        Portal,
        Edges {
            material: 23,
            flags: Flags::ANIMATION_FLAG,
            half_size: IVec3::new(0, 10, 7),
        },
        Transform::from_xyz(-5.0, 0.0, 3.0),
    ));
    */


}

fn update(mut cube: Query<&mut Transform, With<Cube>>, time: Res<Time>) {
    for mut transform in cube.iter_mut() {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}
