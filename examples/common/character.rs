use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_voxel_engine::VoxelPhysics;

const SPEED: f32 = 10.0;
const SENSITIVITY: f32 = 0.006;

#[derive(Component)]
pub struct CharacterEntity {
    pub in_spectator: bool,
    pub grounded: bool,
    pub look_at: Vec3,
    pub up: Vec3,
}

pub struct Character;

impl Plugin for Character {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_character)
            .add_systems(Update, update_character);
    }
}

fn setup_character(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    toggle_grab_cursor(&mut window.single_mut());
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.cursor.grab_mode = match window.cursor.grab_mode {
        CursorGrabMode::Locked => CursorGrabMode::None,
        CursorGrabMode::None => CursorGrabMode::Locked,
        _ => CursorGrabMode::Locked,
    };
    window.cursor.visible = !window.cursor.visible;
}

fn update_character(
    mut character: Query<(&mut Transform, &mut VoxelPhysics, &mut CharacterEntity)>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = window.single_mut();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(&mut window);
    }

    let (mut transform, mut voxel_physics, mut character) = character.single_mut();
    let target_velocity;

    if window.cursor.grab_mode == CursorGrabMode::Locked {
        character.look_at = voxel_physics.portal_rotation * character.look_at;
        character.up = voxel_physics.portal_rotation * character.up;

        // rotation
        let mut mouse_delta = Vec2::new(0.0, 0.0);
        for event in mouse_motion_events.read() {
            mouse_delta += event.delta;
        }
        if mouse_delta != Vec2::ZERO {
            let angle = character.look_at.dot(character.up).acos();
            let max_angle = 0.01;

            // Order is important to prevent unintended roll
            character.look_at = Quat::from_axis_angle(Vec3::Y, -mouse_delta.x * SENSITIVITY)
                * Quat::from_axis_angle(
                    transform.local_x(),
                    (-mouse_delta.y * SENSITIVITY)
                        .min(angle - max_angle)
                        .max(angle + max_angle - std::f32::consts::PI),
                )
                * character.look_at;
        }

        let pos = transform.translation;

        let character_pos = pos + character.look_at;

        transform.look_at(character_pos + character.look_at, character.up);

        // Movement

        let mut input = Vec3::new(
            (keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32) as f32,
            (keys.pressed(KeyCode::Space) as i32 - keys.pressed(KeyCode::ShiftLeft) as i32) as f32,
            (keys.pressed(KeyCode::S) as i32 - keys.pressed(KeyCode::W) as i32) as f32,
        );

        if input != Vec3::ZERO {
            input = input.normalize();
        }

        input *= SPEED;

        if character.in_spectator {
            target_velocity = input.z * transform.local_z()
                + input.x * transform.local_x()
                + input.y * transform.local_y();
        } else {
            if voxel_physics.velocity.y == 0.0 {
                character.grounded = true;
            }

            if input.y > 0.0 && character.grounded {
                voxel_physics.velocity.y += 10.0;
                character.grounded = false;
            }

            voxel_physics.velocity += Vec3::new(0.0, -9.81 * time.delta_seconds(), 0.0);

            let plane_forward = transform.local_x().cross(Vec3::Y).normalize();

            target_velocity = input.z * plane_forward
                + input.x * transform.local_x()
                + voxel_physics.velocity.y * Vec3::Y;
        }
    } else {
        target_velocity = Vec3::splat(0.0);
    }

    let acceleration: f32 = if character.in_spectator {
        0.2
    } else if character.grounded {
        0.2
    } else {
        0.01
    };

    voxel_physics.velocity = lerp(
        voxel_physics.velocity,
        target_velocity,
        acceleration,
        time.delta_seconds(),
    );

    character.up = slerp(
        character.up.normalize(),
        Vec3::Y,
        0.04,
        time.delta_seconds(),
    );
}

fn lerp(i: Vec3, f: Vec3, s: f32, dt: f32) -> Vec3 {
    let s = (1.0 - s).powf(dt * 120.0);
    i * s + f * (1.0 - s)
}

// https://youtu.be/ibkT5ao8kGY
fn slerp(i: Vec3, f: Vec3, s: f32, dt: f32) -> Vec3 {
    let s = (1.0 - s).powf(dt * 120.0);
    let theta = i.dot(f).acos();
    if theta.sin() == 0.0 {
        return i + Vec3::splat(0.00000001);
    }
    ((s * theta).sin() / theta.sin()) * i + (((1.0 - s) * theta).sin() / theta.sin()) * f
}
