use animation::{Edges, Particle, Portal, Velocity};
use bevy::{asset::AssetServerSettings, prelude::*};
use character::CharacterEntity;

mod animation;
mod character;
mod compute;
mod fps_counter;
mod load;
mod trace;
mod ui;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
pub struct Bullet;

pub struct Settings {
    pub spectator: bool,
}

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
        .insert_resource(Settings {
            spectator: true,
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
        .add_system(shoot)
        .add_system(update_velocitys)
        .run();
}

fn shoot(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    character: Query<&Transform, With<CharacterEntity>>,
) {
    let character = character.single();

    if input.pressed(MouseButton::Left) {
        commands.spawn_bundle((
            Transform::from_translation(character.translation).with_rotation(character.rotation),
            Particle { material: 99 },
            Velocity {
                velocity: -character.local_z() * 10.0,
            },
            Bullet,
        ));
    }
}

// world space cordinates are in terms of 4 voxels per meter with 0, 0
// in the world lining up with the center of the voxel world (ie 0, 0, 0 in the render world)
fn setup(mut commands: Commands) {
    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(0, 9, 6),
            normal: Vec3::new(1.0, 0.0, 0.0),
        },
        Edges {
            material: 23,
            half_size: IVec3::new(0, 10, 7),
        },
        Transform::from_xyz(3.0, 2.0, 0.0),
    ));
    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(6, 9, 0),
            normal: Vec3::new(0.0, 0.0, 1.0),
        },
        Edges {
            material: 22,
            half_size: IVec3::new(7, 10, 0),
        },
        Transform::from_xyz(0.0, 2.0, 3.0),
    ));

    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(0, 1, 1),
            normal: Vec3::new(1.0, 0.0, 0.0),
        },
        Edges {
            material: 23,
            half_size: IVec3::new(0, 2, 2),
        },
        Transform::from_xyz(3.0, 5.0, 0.0),
    ));
    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(1, 1, 0),
            normal: Vec3::new(0.0, 0.0, 1.0),
        },
        Edges {
            material: 22,
            half_size: IVec3::new(2, 2, 0),
        },
        Transform::from_xyz(0.0, 5.0, 3.0),
    ));

    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(5, 0, 5),
            normal: Vec3::new(0.0, 1.0, 0.0),
        },
        Edges {
            material: 22,
            half_size: IVec3::new(6, 0, 6),
        },
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
    commands.spawn_bundle((
        Portal {
            half_size: IVec3::new(5, 0, 5),
            normal: Vec3::new(0.0, -1.0, 0.0),
        },
        Edges {
            material: 22,
            half_size: IVec3::new(6, 0, 6),
        },
        Transform::from_xyz(0.0, 7.0, 0.0),
    ));
}

fn update_velocitys(
    mut velocity_query: Query<(&Transform, &mut Velocity, Entity), With<Bullet>>,
    time: Res<Time>,
) {
    velocity_query.par_for_each_mut(8, |(_transform, mut velocity, _entity)| {
        velocity.velocity += Vec3::new(0.0, -9.81 * time.delta_seconds(), 0.0);
        // let e = world_to_render(transform.translation.abs(), uniforms.texture_size);
        // if e.x > 1.0 || e.y > 1.0 || e.z > 1.0 {
        //     to_destroy.push(entity);
        // }
    });
}
