use bevy::{input::mouse::MouseMotion, prelude::*};

const SPEED: f32 = 10.0;
const SENSITIVITY: f32 = 0.004;

#[derive(Component)]
pub struct CharacterEntity {
    pub grounded: bool,
    pub look_at: Vec3,
    pub up: Vec3,
    pub portal1: Entity,
    pub portal2: Entity,
}

pub struct Character;

impl Plugin for Character {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_character)
            .add_system(update_character);
    }
}

fn setup_character(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn update_character(
    mut character: Query<(&mut Transform, &mut super::Velocity, &mut CharacterEntity)>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    settings: Res<super::Settings>,
) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }

    let (mut transform, mut velocity, mut character) = character.single_mut();
    let target_velocity;
    if window.cursor_locked() {
        // movement
        let mut input = Vec3::new(
            (keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32) as f32,
            (keys.pressed(KeyCode::Space) as i32 - keys.pressed(KeyCode::LShift) as i32) as f32,
            (keys.pressed(KeyCode::S) as i32 - keys.pressed(KeyCode::W) as i32) as f32,
        );
        if input != Vec3::ZERO {
            input = input.normalize();
        }
        input *= SPEED;

        if settings.spectator {
            target_velocity = input.z * transform.local_z()
                + input.x * transform.local_x()
                + input.y * transform.local_y();
        } else {
            if velocity.velocity.y == 0.0 {
                character.grounded = true;
            }
            if input.y > 0.0 && character.grounded {
                velocity.velocity.y = 5.0;
                character.grounded = false;
            }
            velocity.velocity += Vec3::new(0.0, -9.81 * time.delta_seconds(), 0.0);

            let plane_forward = transform.local_x().cross(Vec3::Y).normalize();
            target_velocity = input.z * plane_forward
                + input.x * transform.local_x()
                + velocity.velocity.y * Vec3::Y;
        }

        // rotation
        let mut mouse_delta = Vec2::new(0.0, 0.0);
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
        if mouse_delta != Vec2::ZERO {
            let angle = character.look_at.dot(character.up).acos();
            let max_angle = 0.1;

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
        transform.look_at(pos + character.look_at, character.up);
    } else {
        target_velocity = Vec3::splat(0.0);
    }

    let acceleration: f32 = if settings.spectator {
        0.2
    } else if character.grounded {
        0.2
    } else {
        0.01
    };

    velocity.velocity = lerp(
        velocity.velocity,
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
