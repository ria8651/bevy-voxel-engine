use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    prelude::*,
};
use bevy_obj::*;
use bevy_voxel_engine::*;
use character::CharacterEntity;
use std::f32::consts::PI;

mod character;
mod fps_counter;
mod ui;

// zero: normal bullet
// one: orange portal bullet
// two: blue portal bullet
#[derive(Component)]
pub struct Bullet {
    bullet_type: u32,
}

#[derive(Component)]
struct CharacterPortals {
    portal1: Entity,
    portal2: Entity,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(BevyVoxelEnginePlugin)
        .add_plugin(character::Character)
        .add_plugin(ui::UiPlugin)
        .add_plugin(fps_counter::FpsCounter)
        .add_startup_system(setup)
        .add_system(update)
        .add_system(shoot)
        // .add_system(update_velocitys)
        .add_system(spawn_portals);

    let dot = bevy_mod_debugdump::get_render_graph(&mut app);
    std::fs::write("render-graph.dot", dot).expect("Failed to write render-graph.dot");
    println!("Render graph written to render-graph.dot");

    app.run();
}

// world space cordinates are in terms of 4 voxels per meter with 0, 0
// in the world lining up with the center of the voxel world and the edge
// of the world being half of the world size in each direction
fn setup(
    mut commands: Commands,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    asset_server: Res<AssetServer>,
) {
    *load_voxel_world = LoadVoxelWorld::File("assets/monu9.vox".to_string());

    let mut portals = vec![None; 2];
    for i in 0..2 {
        portals[i] = Some(
            commands
                .spawn((
                    VoxelizationBundle {
                        mesh_handle: asset_server.load("models/portal.obj"),
                        transform: Transform::from_xyz(0.0, 100.0, 0.0)
                            .looking_at(Vec3::ZERO, Vec3::Y)
                            .with_scale(Vec3::new(i as f32 * 2.0 - 1.0, 1.0, i as f32 * 2.0 - 1.0)),
                        voxelization_material: VoxelizationMaterial {
                            flags: Flags::ANIMATION_FLAG | Flags::PORTAL_FLAG,
                            ..default()
                        },
                        ..default()
                    },
                    Portal,
                ))
                .with_children(|parent| {
                    // portal border
                    parent.spawn(VoxelizationBundle {
                        mesh_handle: asset_server.load("models/portal_frame.obj"),
                        voxelization_material: VoxelizationMaterial {
                            material: VoxelizationMaterialType::Material(120 + i as u8),
                            flags: Flags::ANIMATION_FLAG | Flags::COLLISION_FLAG,
                        },
                        ..default()
                    });
                })
                .id(),
        );
    }

    // character
    let character_transform = Transform::from_xyz(10.0, 10.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y);
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: PI / 2.0,
        ..default()
    });
    commands
        .spawn((
            VoxelCameraBundle {
                transform: character_transform,
                projection: projection.clone(),
                ..default()
            },
            CharacterEntity {
                in_spectator: false,
                grounded: false,
                look_at: -character_transform.local_z(),
                up: Vec3::new(0.0, 1.0, 0.0),
            },
            CharacterPortals {
                portal1: portals[0].unwrap(),
                portal2: portals[1].unwrap(),
            },
            VoxelPhysics::new(
                Vec3::splat(0.0),
                Vec3::ZERO, // gravity handeled in character.rs
                CollisionEffect::None,
            ),
            BoxCollider {
                half_size: IVec3::new(2, 4, 2),
            },
            BloomSettings::default(),
            Fxaa::default(),
        ))
        .with_children(|parent| {
            // voxelization preview camera
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        is_active: false,
                        priority: 10, // render after the main camera
                        ..default()
                    },
                    projection,
                    ..default()
                },
                BloomSettings::default(),
                Fxaa::default(),
                VoxelizationPreviewCamera,
            ));
        });

    // voxelized mesh
    commands.spawn((
        VoxelizationBundle {
            mesh_handle: asset_server.load("models/suzanne.obj"),
            voxelization_material: VoxelizationMaterial {
                material: VoxelizationMaterialType::Texture(
                    asset_server.load("models/suzanne.png"),
                ),
                flags: Flags::COLLISION_FLAG | Flags::ANIMATION_FLAG,
            },
            transform: Transform::from_scale(Vec3::splat(1.0)).looking_at(Vec3::Z, Vec3::Y),
            ..default()
        },
        Suzanne,
    ));

    // rotated portal
    let pos = vec![Vec3::new(5.0, 0.0, -5.0), Vec3::new(-5.0, 0.0, 5.0)];
    for i in 0..2 {
        commands
            .spawn((
                VoxelizationBundle {
                    mesh_handle: asset_server.load("models/portal.obj"),
                    transform: Transform::from_translation(pos[i])
                        .looking_at(Vec3::ZERO, Vec3::Y)
                        .with_scale(Vec3::new(i as f32 * 2.0 - 1.0, 1.0, i as f32 * 2.0 - 1.0)),
                    voxelization_material: VoxelizationMaterial {
                        flags: Flags::ANIMATION_FLAG | Flags::PORTAL_FLAG,
                        ..default()
                    },
                    ..default()
                },
                Portal,
            ))
            .with_children(|parent| {
                // portal border
                parent.spawn(VoxelizationBundle {
                    mesh_handle: asset_server.load("models/portal_frame.obj"),
                    voxelization_material: VoxelizationMaterial {
                        material: VoxelizationMaterialType::Material(120 + i as u8),
                        flags: Flags::ANIMATION_FLAG | Flags::COLLISION_FLAG,
                    },
                    ..default()
                });
            });
    }
}

#[derive(Component)]
struct VoxelizationPreviewCamera;
#[derive(Component)]
struct Suzanne;

fn update(time: Res<Time>, mut cube: Query<&mut Transform, With<Suzanne>>) {
    for mut transform in cube.iter_mut() {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}

fn shoot(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut character: Query<(&Transform, &mut CharacterEntity)>,
) {
    let (transform, mut character_entity) = character.single_mut();

    // if input.just_pressed(MouseButton::Left) {
    //     commands.spawn((
    //         Transform::from_translation(transform.translation),
    //         Particle { material: 120 },
    //         VoxelPhysics::new(
    //             -transform.local_z() * 50.0,
    //             Vec3::new(0.0, -9.81, 0.0),
    //             CollisionEffect::None,
    //         ),
    //         Bullet { bullet_type: 1 },
    //     ));
    // }
    // if input.just_pressed(MouseButton::Right) {
    //     commands.spawn((
    //         Transform::from_translation(transform.translation),
    //         Particle { material: 121 },
    //         VoxelPhysics::new(
    //             -transform.local_z() * 50.0,
    //             Vec3::new(0.0, -9.81, 0.0),
    //             CollisionEffect::None,
    //         ),
    //         Bullet { bullet_type: 2 },
    //     ));
    // }
    if input.just_pressed(MouseButton::Left) {
        commands.spawn((
            Transform::from_translation(transform.translation),
            Particle { material: 120 },
            VoxelPhysics::new(
                -transform.local_z() * 50.0,
                Vec3::new(0.0, -9.81, 0.0),
                CollisionEffect::SetFlags {
                    radius: 3.0,
                    flags: Flags::SAND_FLAG,
                },
            ),
            Bullet { bullet_type: 0 },
        ));
    }

    if keyboard.just_pressed(KeyCode::P) {
        character_entity.in_spectator = !character_entity.in_spectator;
    }

    if keyboard.just_pressed(KeyCode::B) {
        commands.spawn((
            Transform::from_translation(transform.translation),
            VoxelPhysics::new(
                -transform.local_z() * 10.0,
                Vec3::new(0.0, -9.81, 0.0),
                CollisionEffect::None,
            ),
            Bullet { bullet_type: 0 },
            BoxCollider {
                half_size: IVec3::new(3, 3, 3),
            },
            Box {
                material: 14,
                half_size: IVec3::new(3, 3, 3),
            },
        ));
    }
}

// fn update_velocitys(
//     mut commands: Commands,
//     mut velocity_query: Query<(&Transform, &mut VoxelPhysics, Entity), With<Bullet>>,
//     time: Res<Time>,
// ) {
//     // let to_destroy = ConcurrentQueue::unbounded();
//     // velocity_query.par_for_each_mut(8, |(_transform, mut velocity, _entity)| {
//     //     // velocity.velocity += Vec3::new(0.0, -9.81 * time.delta_seconds(), 0.0);
//     //     // let e = animation::world_to_render(transform.translation.abs(), uniforms.texture_size);
//     //     // if e.x > 1.0 || e.y > 1.0 || e.z > 1.0 {
//     //     //     to_destroy.push(entity).unwrap();
//     //     // }
//     // });

//     // while let Ok(entity) = to_destroy.pop() {
//     //     commands.entity(entity).despawn();
//     // }
// }

fn spawn_portals(
    mut commands: Commands,
    bullet_query: Query<(&Transform, &VoxelPhysics, &Bullet, Entity)>,
    character_query: Query<&CharacterPortals>,
    mut portal_query: Query<&mut Transform, (With<Portal>, Without<Bullet>)>,
) {
    for (transform, velocity, bullet, entity) in bullet_query.iter() {
        if velocity.hit_normal != Vec3::splat(0.0) {
            commands.entity(entity).despawn();

            if bullet.bullet_type == 1 || bullet.bullet_type == 2 {
                let normal = velocity.hit_normal;

                let plane = 1.0 - normal.abs();
                let pos =
                    (transform.translation * plane * VOXELS_PER_METER).floor() / VOXELS_PER_METER;
                let pos = pos + transform.translation * normal.abs();

                let portals = character_query.single();
                let entity = match bullet.bullet_type {
                    1 => portals.portal1,
                    2 => portals.portal2,
                    _ => panic!(),
                };

                let up = if normal.abs() == Vec3::Y {
                    Vec3::Z
                } else {
                    Vec3::Y
                };

                let mut transform = portal_query.get_mut(entity).unwrap();
                transform.translation = pos;
                transform.look_at(pos + normal, up);
            }
        }
    }
}

// fn spawn_world_portals() {
// commands.spawn((
//     Portal {
//         half_size: IVec3::new(0, 9, 6),
//         normal: Vec3::new(1.0, 0.0, 0.0),
//     },
//     Edges {
//         material: 23,
//         half_size: IVec3::new(0, 10, 7),
//     },
//     Transform::from_xyz(3.0, 2.0, 0.0),
// ));
// commands.spawn((
//     Portal {
//         half_size: IVec3::new(6, 9, 0),
//         normal: Vec3::new(0.0, 0.0, 1.0),
//     },
//     Edges {
//         material: 22,
//         half_size: IVec3::new(7, 10, 0),
//     },
//     Transform::from_xyz(0.0, 2.0, 3.0),
// ));

// commands.spawn((
//     Portal {
//         half_size: IVec3::new(0, 1, 1),
//         normal: Vec3::new(1.0, 0.0, 0.0),
//     },
//     Edges {
//         material: 23,
//         half_size: IVec3::new(0, 2, 2),
//     },
//     Transform::from_xyz(3.0, 5.0, 0.0),
// ));
// commands.spawn((
//     Portal {
//         half_size: IVec3::new(1, 1, 0),
//         normal: Vec3::new(0.0, 0.0, 1.0),
//     },
//     Edges {
//         material: 22,
//         half_size: IVec3::new(2, 2, 0),
//     },
//     Transform::from_xyz(0.0, 5.0, 3.0),
// ));

// commands.spawn((
//     Portal {
//         half_size: IVec3::new(5, 0, 5),
//         normal: Vec3::new(0.0, 1.0, 0.0),
//     },
//     Edges {
//         material: 22,
//         half_size: IVec3::new(6, 0, 6),
//     },
//     Transform::from_xyz(0.0, -1.0, 0.0),
// ));
// commands.spawn((
//     Portal {
//         half_size: IVec3::new(5, 0, 5),
//         normal: Vec3::new(0.0, -1.0, 0.0),
//     },
//     Edges {
//         material: 22,
//         half_size: IVec3::new(6, 0, 6),
//     },
//     Transform::from_xyz(0.0, 7.0, 0.0),
// ));
// }
