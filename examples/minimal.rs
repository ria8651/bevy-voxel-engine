use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    prelude::*,
};
use bevy_voxel_engine::{
    BevyVoxelEnginePlugin, Edges, Flags, LoadVoxelWorld, Portal, VoxelCameraBundle,
    VoxelizationBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyVoxelEnginePlugin)
        .add_startup_system(setup)
        .add_system(update)
        .run();
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // voxel world
    *load_voxel_world = LoadVoxelWorld::File("assets/monu9.vox".to_string());

    // camera
    commands.spawn((
        VoxelCameraBundle {
            transform: Transform::from_xyz(5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 1.57,
                ..default()
            }),
            ..default()
        },
        // supports bloom and fxaa
        BloomSettings::default(),
        Fxaa::default(),
    ));

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

    // voxelization
    commands.spawn((
        VoxelizationBundle {
            mesh_handle: meshes.add(Mesh::from(shape::Cube { size: 3.0 })),
            transform: Transform::from_xyz(5.0, 0.0, 5.0),
            ..default()
        },
        Cube,
    ));
}

fn update(mut cube: Query<&mut Transform, With<Cube>>, time: Res<Time>) {
    for mut transform in cube.iter_mut() {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}
